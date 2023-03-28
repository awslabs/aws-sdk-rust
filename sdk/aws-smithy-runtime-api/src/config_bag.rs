/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// This code is functionally equivalent to `Extensions` in the `http` crate. Examples
// have been updated to be more relevant for smithy use, the interface has been made public,
// and the doc comments have been updated to reflect how the config bag is used in smithy-rs.
// Additionally, optimizations around the HTTP use case have been removed in favor or simpler code.

use aws_smithy_http::property_bag::PropertyBag;
use std::any::type_name;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

pub struct ConfigBag {
    head: Layer,
    tail: Option<FrozenConfigBag>,
}

#[derive(Clone)]
pub struct FrozenConfigBag(Arc<ConfigBag>);

impl Deref for FrozenConfigBag {
    type Target = ConfigBag;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

enum Value<T> {
    Set(T),
    ExplicitlyUnset,
}

struct Layer {
    name: &'static str,
    props: PropertyBag,
}

impl FrozenConfigBag {
    pub fn try_modify(self) -> Option<ConfigBag> {
        Arc::try_unwrap(self.0).ok()
    }

    #[must_use]
    pub fn with_open(&self, name: &'static str, next: impl Fn(&mut ConfigBag)) -> ConfigBag {
        let new_layer = Layer {
            name,
            props: PropertyBag::new(),
        };
        let mut bag = ConfigBag {
            head: new_layer,
            tail: Some(self.clone()),
        };
        next(&mut bag);
        bag
    }

    pub fn with(&self, name: &'static str, next: impl Fn(&mut ConfigBag)) -> Self {
        self.with_open(name, next).close()
    }
}

impl ConfigBag {
    pub fn base() -> Self {
        ConfigBag {
            head: Layer {
                name: "base",
                props: Default::default(),
            },
            tail: None,
        }
    }

    pub fn get<T: Send + Sync + Debug + 'static>(&self) -> Option<&T> {
        let mut source = vec![];
        let out = self.sourced_get(&mut source);
        println!("searching for {:?} {:#?}", type_name::<T>(), source);
        out
    }

    pub fn put<T: Send + Sync + Debug + 'static>(&mut self, value: T) -> &mut Self {
        self.head.props.insert(Value::Set(value));
        self
    }

    pub fn unset<T: Send + Sync + 'static>(&mut self) -> &mut Self {
        self.head.props.insert(Value::<T>::ExplicitlyUnset);
        self
    }

    pub fn close(self) -> FrozenConfigBag {
        self.into()
    }

    #[must_use]
    pub fn with(self, name: &'static str, next: impl Fn(&mut ConfigBag)) -> FrozenConfigBag {
        self.close().with_open(name, next).close()
    }

    pub fn sourced_get<T: Send + Sync + Debug + 'static>(
        &self,
        source_trail: &mut Vec<SourceInfo>,
    ) -> Option<&T> {
        let bag = &self.head;
        let inner_item = self
            .tail
            .as_ref()
            .and_then(|bag| bag.sourced_get(source_trail));
        let (item, source) = match bag.props.get::<Value<T>>() {
            Some(Value::ExplicitlyUnset) => (None, SourceInfo::Unset { layer: bag.name }),
            Some(Value::Set(v)) => (
                Some(v),
                SourceInfo::Set {
                    layer: bag.name,
                    value: format!("{:?}", v),
                },
            ),
            None => (inner_item, SourceInfo::Inherit { layer: bag.name }),
        };
        source_trail.push(source);
        item
    }
}

impl From<ConfigBag> for FrozenConfigBag {
    fn from(bag: ConfigBag) -> Self {
        FrozenConfigBag(Arc::new(bag))
    }
}

#[derive(Debug)]
pub enum SourceInfo {
    Set { layer: &'static str, value: String },
    Unset { layer: &'static str },
    Inherit { layer: &'static str },
}

#[cfg(test)]
mod test {
    use super::ConfigBag;

    #[test]
    fn layered_property_bag() {
        #[derive(Debug)]
        struct Prop1;
        #[derive(Debug)]
        struct Prop2;
        let layer_a = |bag: &mut ConfigBag| {
            bag.put(Prop1);
        };

        let layer_b = |bag: &mut ConfigBag| {
            bag.put(Prop2);
        };

        #[derive(Debug)]
        struct Prop3;

        let mut base_bag = ConfigBag::base().with("a", layer_a).with_open("b", layer_b);
        base_bag.put(Prop3);
        let base_bag = base_bag.close();
        assert!(base_bag.get::<Prop1>().is_some());

        #[derive(Debug)]
        struct Prop4;

        let layer_c = |bag: &mut ConfigBag| {
            bag.put(Prop4);
            bag.unset::<Prop3>();
        };

        let final_bag = base_bag.with("c", layer_c);

        assert!(final_bag.get::<Prop4>().is_some());
        assert!(base_bag.get::<Prop4>().is_none());
        assert!(final_bag.get::<Prop1>().is_some());
        assert!(final_bag.get::<Prop2>().is_some());
        // we unset prop3
        assert!(final_bag.get::<Prop3>().is_none());
    }

    #[test]
    fn config_bag() {
        let bag = ConfigBag::base();
        #[derive(Debug)]
        struct Region(&'static str);
        let bag = bag.with("service config", |layer| {
            layer.put(Region("asdf"));
        });

        assert_eq!(bag.get::<Region>().unwrap().0, "asdf");

        #[derive(Debug)]
        struct SigningName(&'static str);
        let operation_config = bag.with("operation", |layer| {
            layer.put(SigningName("s3"));
        });

        assert!(bag.get::<SigningName>().is_none());
        assert_eq!(operation_config.get::<SigningName>().unwrap().0, "s3");

        let mut open_bag = operation_config.with_open("my_custom_info", |_bag| {});
        open_bag.put("foo");
    }
}

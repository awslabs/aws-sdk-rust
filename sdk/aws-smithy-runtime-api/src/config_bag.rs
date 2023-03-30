/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Layered Configuration Bag Structure
//!
//! [`config_bag::ConfigBag`] and [`config_bag::FrozenConfigBag`] are the two representations of a layered configuration structure
//! with the following properties:
//! 1. A new layer of configuration may be applied onto an existing configuration structure without modifying it or taking ownership.
//! 2. No lifetime shenanigans to deal with
use aws_smithy_http::property_bag::PropertyBag;
use std::any::type_name;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

/// Layered Configuration Structure
///
/// [`ConfigBag`] is the "unlocked" form of the bag. Only the top layer of the bag may be unlocked.
#[must_use]
pub struct ConfigBag {
    head: Layer,
    tail: Option<FrozenConfigBag>,
}

/// Layered Configuration Structure
///
/// [`FrozenConfigBag`] is the "locked" form of the bag.
#[derive(Clone)]
#[must_use]
pub struct FrozenConfigBag(Arc<ConfigBag>);

impl Deref for FrozenConfigBag {
    type Target = ConfigBag;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Persist {
    fn layer_name(&self) -> &'static str;
    fn persist(&self, layer: &mut ConfigBag);
}

pub trait Load: Sized {
    fn load(bag: &ConfigBag) -> Option<Self>;
}

pub trait ConfigLayer: Persist + Load {}

enum Value<T> {
    Set(T),
    ExplicitlyUnset,
}

struct Layer {
    name: &'static str,
    props: PropertyBag,
}

fn no_op(_: &mut ConfigBag) {}

impl FrozenConfigBag {
    /// Attempts to convert this bag directly into a [`ConfigBag`] if no other references exist
    ///
    /// This allows modifying the top layer of the bag. [`Self::add_layer`] may be
    /// used to add a new layer to the bag.
    pub fn try_modify(self) -> Option<ConfigBag> {
        Arc::try_unwrap(self.0).ok()
    }

    /// Add a new layer to the config bag
    ///
    /// This is equivalent to calling [`Self::with_fn`] with a no-op function
    ///
    /// # Examples
    /// ```
    /// use aws_smithy_runtime_api::config_bag::ConfigBag;
    /// fn add_more_config(bag: &mut ConfigBag) { /* ... */ }
    /// let bag = ConfigBag::base().with_fn("first layer", |_| { /* add a property */ });
    /// let mut bag = bag.add_layer("second layer");
    /// add_more_config(&mut bag);
    /// let bag = bag.freeze();
    /// ```
    pub fn add_layer(&self, name: &'static str) -> ConfigBag {
        self.with_fn(name, no_op)
    }

    pub fn with(&self, layer: impl Persist) -> ConfigBag {
        self.with_fn(layer.layer_name(), |bag| layer.persist(bag))
    }

    /// Add more items to the config bag
    pub fn with_fn(&self, name: &'static str, next: impl Fn(&mut ConfigBag)) -> ConfigBag {
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

    /// Retrieve the value of type `T` from the bag if exists
    pub fn get<T: Send + Sync + Debug + 'static>(&self) -> Option<&T> {
        let mut source = vec![];
        let out = self.sourced_get(&mut source);
        println!("searching for {:?} {:#?}", type_name::<T>(), source);
        out
    }

    /// Insert `value` into the bag
    pub fn put<T: Send + Sync + Debug + 'static>(&mut self, value: T) -> &mut Self {
        self.head.props.insert(Value::Set(value));
        self
    }

    /// Remove `T` from this bag
    pub fn unset<T: Send + Sync + 'static>(&mut self) -> &mut Self {
        self.head.props.insert(Value::<T>::ExplicitlyUnset);
        self
    }

    /// Freeze this layer by wrapping it in an `Arc`
    ///
    /// This prevents further items from being added to this layer, but additional layers can be
    /// added to the bag.
    pub fn freeze(self) -> FrozenConfigBag {
        self.into()
    }

    /// Add another layer to this configuration bag
    ///
    /// Hint: If you want to re-use this layer, call `freeze` first.
    /// ```
    /// use aws_smithy_runtime_api::config_bag::ConfigBag;
    /// let bag = ConfigBag::base();
    /// let first_layer = bag.with_fn("a", |b: &mut ConfigBag| { b.put("a"); }).freeze();
    /// let second_layer = first_layer.with_fn("other", |b: &mut ConfigBag| { b.put(1i32); });
    /// // The number is only in the second layer
    /// assert_eq!(first_layer.get::<i32>(), None);
    /// assert_eq!(second_layer.get::<i32>(), Some(&1));
    ///
    /// // The string is in both layers
    /// assert_eq!(first_layer.get::<&'static str>(), Some(&"a"));
    /// assert_eq!(second_layer.get::<&'static str>(), Some(&"a"));
    /// ```
    pub fn with_fn(self, name: &'static str, next: impl Fn(&mut ConfigBag)) -> ConfigBag {
        self.freeze().with_fn(name, next)
    }

    pub fn with(self, layer: impl Persist) -> ConfigBag {
        self.freeze().with(layer)
    }

    pub fn add_layer(self, name: &'static str) -> ConfigBag {
        self.freeze().add_layer(name)
    }

    pub fn sourced_get<T: Send + Sync + Debug + 'static>(
        &self,
        source_trail: &mut Vec<SourceInfo>,
    ) -> Option<&T> {
        // todo: optimize so we don't need to compute the source if it's unused
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
    use crate::config_bag::{Load, Persist};

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

        let mut base_bag = ConfigBag::base()
            .with_fn("a", layer_a)
            .with_fn("b", layer_b);
        base_bag.put(Prop3);
        assert!(base_bag.get::<Prop1>().is_some());

        #[derive(Debug)]
        struct Prop4;

        let layer_c = |bag: &mut ConfigBag| {
            bag.put(Prop4);
            bag.unset::<Prop3>();
        };

        let base_bag = base_bag.freeze();
        let final_bag = base_bag.with_fn("c", layer_c);

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
        let bag = bag.with_fn("service config", |layer: &mut ConfigBag| {
            layer.put(Region("asdf"));
        });

        assert_eq!(bag.get::<Region>().unwrap().0, "asdf");

        #[derive(Debug)]
        struct SigningName(&'static str);
        let bag = bag.freeze();
        let operation_config = bag.with_fn("operation", |layer: &mut ConfigBag| {
            layer.put(SigningName("s3"));
        });

        assert!(bag.get::<SigningName>().is_none());
        assert_eq!(operation_config.get::<SigningName>().unwrap().0, "s3");

        let mut open_bag = operation_config.with_fn("my_custom_info", |_bag: &mut ConfigBag| {});
        open_bag.put("foo");
    }

    #[test]
    fn persist_trait() {
        #[derive(Debug, Eq, PartialEq, Clone)]
        struct MyConfig {
            a: bool,
            b: String,
        }

        #[derive(Debug)]
        struct A(bool);
        #[derive(Debug)]
        struct B(String);

        impl Persist for MyConfig {
            fn layer_name(&self) -> &'static str {
                "my_config"
            }

            fn persist(&self, layer: &mut ConfigBag) {
                layer.put(A(self.a));
                layer.put(B(self.b.clone()));
            }
        }
        impl Load for MyConfig {
            fn load(bag: &ConfigBag) -> Option<Self> {
                Some(MyConfig {
                    a: bag.get::<A>().unwrap().0,
                    b: bag.get::<B>().unwrap().0.clone(),
                })
            }
        }

        let conf = MyConfig {
            a: true,
            b: "hello!".to_string(),
        };

        let bag = ConfigBag::base().with(conf.clone());

        assert_eq!(MyConfig::load(&bag), Some(conf));
    }
}

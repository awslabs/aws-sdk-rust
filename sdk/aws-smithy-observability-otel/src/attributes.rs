/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Utilities to transform back and forth from Smithy Observability [Attributes] to
//! OTel [KeyValue]s.

use std::ops::Deref;

use aws_smithy_observability::{AttributeValue, Attributes};
use opentelemetry::{KeyValue, Value};

pub(crate) struct AttributesWrap(Attributes);
impl AttributesWrap {
    pub(crate) fn new(inner: Attributes) -> Self {
        Self(inner)
    }
}
impl Deref for AttributesWrap {
    type Target = Attributes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn kv_from_option_attr(input: Option<&Attributes>) -> Vec<KeyValue> {
    input
        .map(|attr| AttributesWrap::new(attr.clone()))
        .unwrap_or(AttributesWrap::new(Attributes::new()))
        .into()
}

#[allow(dead_code)]
pub(crate) fn option_attr_from_kv(input: &[KeyValue]) -> Option<Attributes> {
    if input.is_empty() {
        return None;
    }

    Some(AttributesWrap::from(input).0)
}

impl From<AttributesWrap> for Vec<KeyValue> {
    fn from(value: AttributesWrap) -> Self {
        value
            .0
            .into_attributes()
            .map(|(k, v)| {
                KeyValue::new(
                    k,
                    match v {
                        AttributeValue::I64(val) => Value::I64(val),
                        AttributeValue::F64(val) => Value::F64(val),
                        AttributeValue::String(val) => Value::String(val.into()),
                        AttributeValue::Bool(val) => Value::Bool(val),
                        _ => Value::String("UNSUPPORTED ATTRIBUTE VALUE TYPE".into()),
                    },
                )
            })
            .collect::<Vec<KeyValue>>()
    }
}

impl From<&[KeyValue]> for AttributesWrap {
    fn from(value: &[KeyValue]) -> Self {
        let mut attrs = Attributes::new();

        value.iter().for_each(|kv| {
            attrs.set(
                kv.key.clone(),
                match &kv.value {
                    Value::Bool(val) => AttributeValue::Bool(*val),
                    Value::I64(val) => AttributeValue::I64(*val),
                    Value::F64(val) => AttributeValue::F64(*val),
                    Value::String(val) => AttributeValue::String(val.clone().into()),
                    Value::Array(_) => {
                        AttributeValue::String("UNSUPPORTED ATTRIBUTE VALUE TYPE".into())
                    }
                },
            )
        });

        AttributesWrap(attrs)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use aws_smithy_observability::{AttributeValue, Attributes};
    use opentelemetry::Value;

    #[test]
    fn attr_to_kv() {
        let mut attrs = Attributes::new();
        attrs.set("I64", AttributeValue::I64(64));
        attrs.set("F64", AttributeValue::F64(64.0));
        attrs.set("String", AttributeValue::String("I AM A STRING".into()));
        attrs.set("Bool", AttributeValue::Bool(true));

        let kv = kv_from_option_attr(Some(&attrs));

        let kv_map: HashMap<String, Value> = kv
            .into_iter()
            .map(|kv| (kv.key.to_string(), kv.value))
            .collect();

        assert_eq!(kv_map.get("I64").unwrap(), &Value::I64(64));
        assert_eq!(kv_map.get("F64").unwrap(), &Value::F64(64.0));
        assert_eq!(
            kv_map.get("String").unwrap(),
            &Value::String("I AM A STRING".into())
        );
        assert_eq!(kv_map.get("Bool").unwrap(), &Value::Bool(true));
    }

    #[test]
    fn kv_to_attr() {
        let kvs: Vec<KeyValue> = vec![
            KeyValue::new("Bool", Value::Bool(true)),
            KeyValue::new("String", Value::String("I AM A STRING".into())),
            KeyValue::new("I64", Value::I64(64)),
            KeyValue::new("F64", Value::F64(64.0)),
        ];

        let attrs = option_attr_from_kv(&kvs).unwrap();
        assert_eq!(attrs.get("Bool").unwrap(), &AttributeValue::Bool(true));
        assert_eq!(
            attrs.get("String").unwrap(),
            &AttributeValue::String("I AM A STRING".into())
        );
        assert_eq!(attrs.get("I64").unwrap(), &AttributeValue::I64(64));
        assert_eq!(attrs.get("F64").unwrap(), &AttributeValue::F64(64.0));
    }
}

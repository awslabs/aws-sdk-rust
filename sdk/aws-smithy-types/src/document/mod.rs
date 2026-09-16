/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[cfg(all(aws_sdk_unstable, feature = "serde-deserialize"))]
mod de;
#[cfg(any(
    all(aws_sdk_unstable, feature = "serde-deserialize"),
    all(aws_sdk_unstable, feature = "serde-serialize")
))]
mod doc_error;
#[cfg(all(aws_sdk_unstable, feature = "serde-serialize"))]
mod ser;

#[cfg(all(aws_sdk_unstable, feature = "serde-deserialize"))]
pub use de::from_document;
#[cfg(any(
    all(aws_sdk_unstable, feature = "serde-deserialize"),
    all(aws_sdk_unstable, feature = "serde-serialize")
))]
pub use doc_error::DocError;
#[cfg(all(aws_sdk_unstable, feature = "serde-serialize"))]
pub use ser::to_document;

use crate::Number;
use std::borrow::Cow;
use std::collections::HashMap;

#[cfg(any(
    all(aws_sdk_unstable, feature = "serde-deserialize"),
    all(aws_sdk_unstable, feature = "serde-serialize")
))]
use serde;

/// Document Type
///
/// Document types represents protocol-agnostic open content that is accessed like JSON data.
/// Open content is useful for modeling unstructured data that has no schema, data that can't be
/// modeled using rigid types, or data that has a schema that evolves outside of the purview of a model.
/// The serialization format of a document is an implementation detail of a protocol.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    all(aws_sdk_unstable, feature = "serde-serialize"),
    derive(serde::Serialize)
)]
#[cfg_attr(
    all(aws_sdk_unstable, feature = "serde-deserialize"),
    derive(serde::Deserialize)
)]
#[cfg_attr(
    any(
        all(aws_sdk_unstable, feature = "serde-deserialize"),
        all(aws_sdk_unstable, feature = "serde-serialize")
    ),
    serde(untagged)
)]
pub enum Document {
    /// JSON object
    Object(HashMap<String, Document>),
    /// JSON array
    Array(Vec<Document>),
    /// JSON number
    Number(Number),
    /// JSON string
    String(String),
    /// JSON boolean
    Bool(bool),
    /// JSON null
    Null,
}

impl Document {
    /// Returns the inner map value if this `Document` is an object.
    pub fn as_object(&self) -> Option<&HashMap<String, Document>> {
        if let Self::Object(object) = self {
            Some(object)
        } else {
            None
        }
    }

    /// Returns the mutable inner map value if this `Document` is an object.
    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, Document>> {
        if let Self::Object(object) = self {
            Some(object)
        } else {
            None
        }
    }

    /// Returns the inner array value if this `Document` is an array.
    pub fn as_array(&self) -> Option<&Vec<Document>> {
        if let Self::Array(array) = self {
            Some(array)
        } else {
            None
        }
    }

    /// Returns the mutable inner array value if this `Document` is an array.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Document>> {
        if let Self::Array(array) = self {
            Some(array)
        } else {
            None
        }
    }

    /// Returns the inner number value if this `Document` is a number.
    pub fn as_number(&self) -> Option<&Number> {
        if let Self::Number(number) = self {
            Some(number)
        } else {
            None
        }
    }

    /// Returns the inner string value if this `Document` is a string.
    pub fn as_string(&self) -> Option<&str> {
        if let Self::String(string) = self {
            Some(string)
        } else {
            None
        }
    }

    /// Returns the inner boolean value if this `Document` is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(boolean) = self {
            Some(*boolean)
        } else {
            None
        }
    }

    /// Returns `Some(())` if this `Document` is a null.
    pub fn as_null(&self) -> Option<()> {
        if let Self::Null = self {
            Some(())
        } else {
            None
        }
    }

    /// Returns `true` if this `Document` is an object.
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Returns `true` if this `Document` is an array.
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// Returns `true` if this `Document` is a number.
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    /// Returns `true` if this `Document` is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns `true` if this `Document` is a bool.
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    /// Returns `true` if this `Document` is a boolean.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

/// The default value is `Document::Null`.
impl Default for Document {
    fn default() -> Self {
        Self::Null
    }
}

impl From<bool> for Document {
    fn from(value: bool) -> Self {
        Document::Bool(value)
    }
}

impl<'a> From<&'a str> for Document {
    fn from(value: &'a str) -> Self {
        Document::String(value.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for Document {
    fn from(value: Cow<'a, str>) -> Self {
        Document::String(value.into_owned())
    }
}

impl From<String> for Document {
    fn from(value: String) -> Self {
        Document::String(value)
    }
}

impl From<Vec<Document>> for Document {
    fn from(values: Vec<Document>) -> Self {
        Document::Array(values)
    }
}

impl From<HashMap<String, Document>> for Document {
    fn from(values: HashMap<String, Document>) -> Self {
        Document::Object(values)
    }
}

impl From<u64> for Document {
    fn from(value: u64) -> Self {
        Document::Number(Number::PosInt(value))
    }
}

impl From<i64> for Document {
    fn from(value: i64) -> Self {
        Document::Number(Number::NegInt(value))
    }
}

impl From<i32> for Document {
    fn from(value: i32) -> Self {
        Document::Number(Number::NegInt(value as i64))
    }
}

impl From<f64> for Document {
    fn from(value: f64) -> Self {
        Document::Number(Number::Float(value))
    }
}

impl From<Number> for Document {
    fn from(value: Number) -> Self {
        Document::Number(value)
    }
}

impl<T> From<Option<T>> for Document
where
    Document: From<T>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(inner) => inner.into(),
            None => Document::Null,
        }
    }
}

/* ANCHOR END: document */

#[cfg(test)]
#[cfg(all(
    aws_sdk_unstable,
    feature = "serde-serialize",
    feature = "serde-deserialize"
))]
mod test {
    use super::{from_document, to_document, Document};
    use crate::Number;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    /// Helper: serialize a value to Document and verify the result.
    fn test_to_document_ok<T>(cases: &[(T, Document)])
    where
        T: Serialize + std::fmt::Debug,
    {
        for (value, expected) in cases {
            let doc = to_document(value).unwrap();
            assert_eq!(&doc, expected, "to_document({:?})", value);
        }
    }

    /// Helper: round-trip T → Document → T.
    fn test_roundtrip<T>(cases: &[T])
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug + Clone,
    {
        for value in cases {
            let doc = to_document(value).unwrap();
            let roundtripped: T = from_document(doc).unwrap();
            assert_eq!(&roundtripped, value, "roundtrip failed for {:?}", value);
        }
    }

    // ========================================================================
    // Null / Unit
    // ========================================================================

    #[test]
    fn test_null() {
        test_to_document_ok(&[((), Document::Null)]);

        let v: () = from_document(Document::Null).unwrap();
        assert_eq!(v, ());
    }

    // ========================================================================
    // Booleans
    // ========================================================================

    #[test]
    fn test_bool() {
        test_to_document_ok(&[(true, Document::Bool(true)), (false, Document::Bool(false))]);
        test_roundtrip(&[true, false]);
    }

    // ========================================================================
    // Unsigned integers
    // ========================================================================

    #[test]
    fn test_u8() {
        test_to_document_ok(&[
            (0u8, Document::Number(Number::PosInt(0))),
            (u8::MAX, Document::Number(Number::PosInt(u8::MAX as u64))),
        ]);
        test_roundtrip(&[0u8, 1, 127, u8::MAX]);
    }

    #[test]
    fn test_u16() {
        test_to_document_ok(&[
            (0u16, Document::Number(Number::PosInt(0))),
            (u16::MAX, Document::Number(Number::PosInt(u16::MAX as u64))),
        ]);
        test_roundtrip(&[0u16, 1, u16::MAX]);
    }

    #[test]
    fn test_u32() {
        test_to_document_ok(&[
            (0u32, Document::Number(Number::PosInt(0))),
            (u32::MAX, Document::Number(Number::PosInt(u32::MAX as u64))),
        ]);
        test_roundtrip(&[0u32, 1, u32::MAX]);
    }

    #[test]
    fn test_u64() {
        test_to_document_ok(&[
            (0u64, Document::Number(Number::PosInt(0))),
            (u64::MAX, Document::Number(Number::PosInt(u64::MAX))),
        ]);
        test_roundtrip(&[0u64, 1, u64::MAX]);
    }

    // ========================================================================
    // Signed integers
    // ========================================================================

    #[test]
    fn test_i8() {
        test_to_document_ok(&[
            (0i8, Document::Number(Number::PosInt(0))),
            (-1i8, Document::Number(Number::NegInt(-1))),
            (i8::MIN, Document::Number(Number::NegInt(i8::MIN as i64))),
            (i8::MAX, Document::Number(Number::PosInt(i8::MAX as u64))),
        ]);
        test_roundtrip(&[0i8, -1, 1, i8::MIN, i8::MAX]);
    }

    #[test]
    fn test_i16() {
        test_to_document_ok(&[
            (0i16, Document::Number(Number::PosInt(0))),
            (i16::MIN, Document::Number(Number::NegInt(i16::MIN as i64))),
            (i16::MAX, Document::Number(Number::PosInt(i16::MAX as u64))),
        ]);
        test_roundtrip(&[0i16, -1, i16::MIN, i16::MAX]);
    }

    #[test]
    fn test_i32() {
        test_to_document_ok(&[
            (0i32, Document::Number(Number::PosInt(0))),
            (i32::MIN, Document::Number(Number::NegInt(i32::MIN as i64))),
            (i32::MAX, Document::Number(Number::PosInt(i32::MAX as u64))),
        ]);
        test_roundtrip(&[0i32, -1, i32::MIN, i32::MAX]);
    }

    #[test]
    fn test_i64() {
        test_to_document_ok(&[
            (0i64, Document::Number(Number::PosInt(0))),
            (-1i64, Document::Number(Number::NegInt(-1))),
            (i64::MIN, Document::Number(Number::NegInt(i64::MIN))),
            (i64::MAX, Document::Number(Number::PosInt(i64::MAX as u64))),
        ]);
        test_roundtrip(&[0i64, -1, i64::MIN, i64::MAX]);
    }

    // ========================================================================
    // Floats
    // ========================================================================

    #[test]
    fn test_f32() {
        test_to_document_ok(&[
            (0.0f32, Document::Number(Number::Float(0.0))),
            (3.5f32, Document::Number(Number::Float(3.5))),
            (-1.5f32, Document::Number(Number::Float(-1.5))),
        ]);
        test_roundtrip(&[0.0f32, 3.5, -1.5, f32::MIN, f32::MAX]);
    }

    #[test]
    fn test_f64() {
        test_to_document_ok(&[
            (0.0f64, Document::Number(Number::Float(0.0))),
            (3.1f64, Document::Number(Number::Float(3.1))),
            (-1.5f64, Document::Number(Number::Float(-1.5))),
            (f64::MIN, Document::Number(Number::Float(f64::MIN))),
            (f64::MAX, Document::Number(Number::Float(f64::MAX))),
            (f64::EPSILON, Document::Number(Number::Float(f64::EPSILON))),
        ]);
        test_roundtrip(&[0.0f64, 3.1, -1.5, 0.5, f64::MIN, f64::MAX]);
    }

    #[test]
    fn test_nonfinite_floats() {
        // NaN, +Inf, -Inf all serialize to the float Document representation
        let doc = to_document(&f64::NAN).unwrap();
        match doc {
            Document::Number(Number::Float(v)) => assert!(v.is_nan()),
            other => panic!("expected NaN float, got {:?}", other),
        }

        let doc = to_document(&f64::INFINITY).unwrap();
        assert_eq!(doc, Document::Number(Number::Float(f64::INFINITY)));

        let doc = to_document(&f64::NEG_INFINITY).unwrap();
        assert_eq!(doc, Document::Number(Number::Float(f64::NEG_INFINITY)));
    }

    // ========================================================================
    // Strings
    // ========================================================================

    #[test]
    fn test_string() {
        test_to_document_ok(&[
            (String::new(), Document::String(String::new())),
            ("hello".to_owned(), Document::String("hello".to_owned())),
            (
                "with\nnewline".to_owned(),
                Document::String("with\nnewline".to_owned()),
            ),
            (
                "unicode: \u{1F600}".to_owned(),
                Document::String("unicode: \u{1F600}".to_owned()),
            ),
        ]);
        test_roundtrip(&[
            String::new(),
            "foo".to_owned(),
            "bar\tbaz".to_owned(),
            "\u{3A3}".to_owned(),
        ]);
    }

    #[test]
    fn test_str_ref() {
        let doc = to_document(&"borrowed str").unwrap();
        assert_eq!(doc, Document::String("borrowed str".to_owned()));
    }

    #[test]
    fn test_char() {
        let doc = to_document(&'a').unwrap();
        assert_eq!(doc, Document::String("a".to_owned()));

        let doc = to_document(&'\u{1F600}').unwrap();
        assert_eq!(doc, Document::String("\u{1F600}".to_owned()));
    }

    // ========================================================================
    // Option
    // ========================================================================

    #[test]
    fn test_option() {
        test_to_document_ok(&[
            (None::<String>, Document::Null),
            (
                Some("jodhpurs".to_owned()),
                Document::String("jodhpurs".to_owned()),
            ),
        ]);
        test_to_document_ok(&[
            (None::<u32>, Document::Null),
            (Some(42u32), Document::Number(Number::PosInt(42))),
        ]);
        test_roundtrip(&[None::<u32>, Some(5), Some(0)]);
        test_roundtrip(&[None::<String>, Some("x".to_owned())]);
    }

    // ========================================================================
    // Sequences / Arrays
    // ========================================================================

    #[test]
    fn test_vec_empty() {
        let doc = to_document(&Vec::<i32>::new()).unwrap();
        assert_eq!(doc, Document::Array(vec![]));

        let v: Vec<i32> = from_document(Document::Array(vec![])).unwrap();
        assert_eq!(v, Vec::<i32>::new());
    }

    #[test]
    fn test_vec_integers() {
        test_to_document_ok(&[(
            vec![1u64, 2, 3],
            Document::Array(vec![
                Document::Number(Number::PosInt(1)),
                Document::Number(Number::PosInt(2)),
                Document::Number(Number::PosInt(3)),
            ]),
        )]);
        test_roundtrip(&[vec![1i32, -2, 3], vec![], vec![0]]);
    }

    #[test]
    fn test_vec_mixed_via_document() {
        // Vec<Document> allows mixed types
        let mixed = vec![
            Document::Bool(true),
            Document::Null,
            Document::String("foo".to_owned()),
            Document::Number(Number::PosInt(42)),
        ];
        let doc = Document::Array(mixed.clone());
        let roundtripped: Vec<Document> = from_document(doc.clone()).unwrap();
        assert_eq!(roundtripped, mixed);
    }

    #[test]
    fn test_nested_vec() {
        test_roundtrip(&[
            vec![vec![1u32, 2], vec![], vec![3]],
            vec![vec![], vec![], vec![]],
        ]);
    }

    #[test]
    fn test_tuple() {
        let doc = to_document(&(5u32,)).unwrap();
        assert_eq!(
            doc,
            Document::Array(vec![Document::Number(Number::PosInt(5))])
        );

        let doc = to_document(&(1u32, "abc", true)).unwrap();
        assert_eq!(
            doc,
            Document::Array(vec![
                Document::Number(Number::PosInt(1)),
                Document::String("abc".to_owned()),
                Document::Bool(true),
            ])
        );

        test_roundtrip(&[(1u32, 2u32), (0, u32::MAX)]);
        test_roundtrip(&[(1i32, "hello".to_owned(), true)]);
    }

    // ========================================================================
    // Maps / Objects
    // ========================================================================

    #[test]
    fn test_map_empty() {
        let map: HashMap<String, u32> = HashMap::new();
        let doc = to_document(&map).unwrap();
        assert_eq!(doc, Document::Object(HashMap::new()));
        test_roundtrip(&[HashMap::<String, u32>::new()]);
    }

    #[test]
    fn test_map_string_keys() {
        let mut map = HashMap::new();
        map.insert("a".to_owned(), 1u32);
        map.insert("b".to_owned(), 2u32);
        test_roundtrip(&[map]);
    }

    #[test]
    fn test_map_integer_keys() {
        // Integer keys get serialized as their string representation
        let mut map = HashMap::new();
        map.insert(1u32, "one".to_owned());
        map.insert(2u32, "two".to_owned());

        let doc = to_document(&map).unwrap();
        assert!(doc.is_object());
        let obj = doc.as_object().unwrap();
        assert!(obj.contains_key("1") || obj.contains_key("2"));
    }

    #[test]
    fn test_nested_map() {
        let mut inner = HashMap::new();
        inner.insert("x".to_owned(), 10u32);

        let mut outer = HashMap::new();
        outer.insert("inner".to_owned(), inner.clone());

        test_roundtrip(&[outer]);
    }

    // ========================================================================
    // Structs
    // ========================================================================

    #[test]
    fn test_struct() {
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
        struct Inner {
            a: (),
            b: usize,
            c: Vec<String>,
        }

        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
        struct Outer {
            inner: Vec<Inner>,
        }

        let outer = Outer {
            inner: vec![Inner {
                a: (),
                b: 2,
                c: vec!["abc".to_owned(), "xyz".to_owned()],
            }],
        };

        let doc = to_document(&outer).unwrap();
        assert!(doc.is_object());
        let roundtripped: Outer = from_document(doc).unwrap();
        assert_eq!(outer, roundtripped);

        // Empty inner
        test_roundtrip(&[Outer { inner: vec![] }]);
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
        struct Wrapper(u32);

        test_to_document_ok(&[(Wrapper(123), Document::Number(Number::PosInt(123)))]);
        test_roundtrip(&[Wrapper(0), Wrapper(u32::MAX)]);
    }

    #[test]
    fn test_unit_struct() {
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
        struct Unit;

        test_to_document_ok(&[(Unit, Document::Null)]);
        let v: Unit = from_document(Document::Null).unwrap();
        assert_eq!(v, Unit);
    }

    // ========================================================================
    // Enums
    // ========================================================================

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    enum Animal {
        Dog,
        Frog(String, Vec<isize>),
        Cat { age: usize, name: String },
        AntHive(Vec<String>),
    }

    #[test]
    fn test_enum_unit_variant() {
        let doc = to_document(&Animal::Dog).unwrap();
        assert_eq!(doc, Document::String("Dog".to_owned()));
        test_roundtrip(&[Animal::Dog]);
    }

    #[test]
    fn test_enum_tuple_variant() {
        let frog = Animal::Frog("Henry".to_owned(), vec![349, 102]);
        let doc = to_document(&frog).unwrap();
        assert!(doc.is_object());
        let obj = doc.as_object().unwrap();
        assert!(obj.contains_key("Frog"));
        test_roundtrip(&[
            Animal::Frog("Henry".to_owned(), vec![]),
            Animal::Frog("Henry".to_owned(), vec![349, 102]),
        ]);
    }

    #[test]
    fn test_enum_struct_variant() {
        let cat = Animal::Cat {
            age: 5,
            name: "Kate".to_owned(),
        };
        let doc = to_document(&cat).unwrap();
        assert!(doc.is_object());
        let obj = doc.as_object().unwrap();
        assert!(obj.contains_key("Cat"));
        test_roundtrip(&[cat]);
    }

    #[test]
    fn test_enum_newtype_variant() {
        let hive = Animal::AntHive(vec!["Bob".to_owned(), "Stuart".to_owned()]);
        test_roundtrip(&[hive]);
    }

    // ========================================================================
    // Bytes
    // ========================================================================

    #[test]
    fn test_bytes() {
        // Bytes serialize as an array of numbers
        let data: &[u8] = &[1, 2, 3];
        let doc = to_document(&data).unwrap();
        assert_eq!(
            doc,
            Document::Array(vec![
                Document::Number(Number::PosInt(1)),
                Document::Number(Number::PosInt(2)),
                Document::Number(Number::PosInt(3)),
            ])
        );

        let empty: &[u8] = &[];
        let doc = to_document(&empty).unwrap();
        assert_eq!(doc, Document::Array(vec![]));
    }

    // ========================================================================
    // Error cases
    // ========================================================================

    #[test]
    fn test_deserialize_wrong_type() {
        let result = from_document::<bool>(Document::String("not a bool".to_owned()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("invalid type"),
            "unexpected error message: {}",
            err
        );
    }

    #[test]
    fn test_deserialize_missing_field() {
        #[derive(Debug, Deserialize)]
        struct Required {
            #[allow(dead_code)]
            x: u32,
        }

        let doc = Document::Object(HashMap::new());
        let result = from_document::<Required>(doc);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing field"));
    }

    #[test]
    fn test_serialize_non_string_map_key_rejected() {
        // Map keys that cannot be coerced to strings should fail
        use std::collections::HashMap;
        let mut map: HashMap<Option<u32>, u32> = HashMap::new();
        map.insert(None, 1);

        let result = to_document(&map);
        assert!(result.is_err());
    }

    // ========================================================================
    // serde_json compatibility (existing test)
    // ========================================================================

    #[test]
    fn test_serde_json_compatibility() {
        let mut map: HashMap<String, Document> = HashMap::new();
        map.insert("hello".into(), "world".to_string().into());
        map.insert("pos_int".into(), Document::Number(Number::PosInt(1).into()));
        map.insert(
            "neg_int".into(),
            Document::Number(Number::NegInt(-1).into()),
        );
        map.insert(
            "float".into(),
            Document::Number(Number::Float(0.1 + 0.2).into()),
        );
        map.insert("true".into(), true.into());
        map.insert("false".into(), false.into());
        map.insert(
            "array".into(),
            vec![
                map.clone().into(),
                "hello-world".to_string().into(),
                true.into(),
                false.into(),
            ]
            .into(),
        );
        map.insert("map".into(), map.clone().into());
        map.insert("null".into(), Document::Null);
        let obj = Document::Object(map);

        let target_file = include_str!("../../test_data/serialize_document.json");
        let json: Result<serde_json::Value, _> = serde_json::from_str(target_file);
        assert_eq!(serde_json::to_value(&obj).unwrap(), json.unwrap());
        let doc: Result<Document, _> = serde_json::from_str(target_file);
        assert_eq!(obj, doc.unwrap());
    }
}

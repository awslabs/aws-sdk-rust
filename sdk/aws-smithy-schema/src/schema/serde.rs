/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Serialization and deserialization interfaces for the Smithy data model.

mod deserializer;
pub mod error;
mod serializer;

pub use deserializer::{capped_container_size, ShapeDeserializer, MAX_CONTAINER_PREALLOC};
pub use error::SerdeError;
pub use serializer::{SerializableStruct, ShapeSerializer};

#[cfg(test)]
mod test {
    use crate::serde::{SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer};
    use crate::{prelude::*, Schema};

    // Mock serializer for testing
    struct MockSerializer {
        output: Vec<String>,
    }

    impl MockSerializer {
        fn finish(self) -> Vec<String> {
            self.output
        }
    }

    impl ShapeSerializer for MockSerializer {
        fn write_struct(
            &mut self,
            schema: &Schema<'_>,
            value: &dyn SerializableStruct,
        ) -> Result<(), SerdeError> {
            self.output
                .push(format!("struct({})", schema.shape_id().as_str()));
            value.serialize_members(self)?;
            self.output.push("end_struct".to_string());
            Ok(())
        }

        fn write_list(
            &mut self,
            schema: &Schema<'_>,
            write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            self.output
                .push(format!("list({})", schema.shape_id().as_str()));
            write_elements(self)?;
            self.output.push("end_list".to_string());
            Ok(())
        }

        fn write_map(
            &mut self,
            schema: &Schema<'_>,
            write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            self.output
                .push(format!("map({})", schema.shape_id().as_str()));
            write_entries(self)?;
            self.output.push("end_map".to_string());
            Ok(())
        }

        fn write_boolean(&mut self, _schema: &Schema<'_>, value: bool) -> Result<(), SerdeError> {
            self.output.push(format!("bool({})", value));
            Ok(())
        }

        fn write_byte(&mut self, _schema: &Schema<'_>, value: i8) -> Result<(), SerdeError> {
            self.output.push(format!("byte({})", value));
            Ok(())
        }

        fn write_short(&mut self, _schema: &Schema<'_>, value: i16) -> Result<(), SerdeError> {
            self.output.push(format!("short({})", value));
            Ok(())
        }

        fn write_integer(&mut self, _schema: &Schema<'_>, value: i32) -> Result<(), SerdeError> {
            self.output.push(format!("int({})", value));
            Ok(())
        }

        fn write_long(&mut self, _schema: &Schema<'_>, value: i64) -> Result<(), SerdeError> {
            self.output.push(format!("long({})", value));
            Ok(())
        }

        fn write_float(&mut self, _schema: &Schema<'_>, value: f32) -> Result<(), SerdeError> {
            self.output.push(format!("float({})", value));
            Ok(())
        }

        fn write_double(&mut self, _schema: &Schema<'_>, value: f64) -> Result<(), SerdeError> {
            self.output.push(format!("double({})", value));
            Ok(())
        }

        fn write_big_integer(
            &mut self,
            _schema: &Schema<'_>,
            value: &aws_smithy_types::BigInteger,
        ) -> Result<(), SerdeError> {
            self.output.push(format!("bigint({})", value.as_ref()));
            Ok(())
        }

        fn write_big_decimal(
            &mut self,
            _schema: &Schema<'_>,
            value: &aws_smithy_types::BigDecimal,
        ) -> Result<(), SerdeError> {
            self.output.push(format!("bigdec({})", value.as_ref()));
            Ok(())
        }

        fn write_string(&mut self, _schema: &Schema<'_>, value: &str) -> Result<(), SerdeError> {
            self.output.push(format!("string({})", value));
            Ok(())
        }

        fn write_blob(
            &mut self,
            _schema: &Schema<'_>,
            value: aws_smithy_types::Blob,
        ) -> Result<(), SerdeError> {
            self.output
                .push(format!("blob({} bytes)", value.as_ref().len()));
            Ok(())
        }

        fn write_timestamp(
            &mut self,
            _schema: &Schema<'_>,
            value: &aws_smithy_types::DateTime,
        ) -> Result<(), SerdeError> {
            self.output.push(format!("timestamp({})", value));
            Ok(())
        }

        fn write_document(
            &mut self,
            _schema: &Schema<'_>,
            _value: &aws_smithy_types::Document,
        ) -> Result<(), SerdeError> {
            self.output.push("document".to_string());
            Ok(())
        }

        fn write_null(&mut self, _schema: &Schema<'_>) -> Result<(), SerdeError> {
            self.output.push("null".to_string());
            Ok(())
        }
    }

    // Mock deserializer for testing
    struct MockDeserializer {
        values: Vec<String>,
        index: usize,
    }

    impl MockDeserializer {
        fn new(values: Vec<String>) -> Self {
            Self { values, index: 0 }
        }
    }

    impl ShapeDeserializer for MockDeserializer {
        fn read_struct(
            &mut self,
            _schema: &Schema<'_>,
            consumer: &mut dyn FnMut(
                &Schema<'_>,
                &mut dyn ShapeDeserializer,
            ) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            // Simulate reading 2 members
            consumer(&STRING, self)?;
            consumer(&INTEGER, self)?;
            Ok(())
        }

        fn read_list(
            &mut self,
            _schema: &Schema<'_>,
            consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            // Simulate reading 3 elements
            for _ in 0..3 {
                consumer(self)?;
            }
            Ok(())
        }

        fn read_map(
            &mut self,
            _schema: &Schema<'_>,
            consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            // Simulate reading 2 entries
            consumer("key1".to_string(), self)?;
            consumer("key2".to_string(), self)?;
            Ok(())
        }

        fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
            Ok(true)
        }

        fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
            Ok(42)
        }

        fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
            Ok(1000)
        }

        fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
            Ok(123456)
        }

        fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
            Ok(9876543210)
        }

        fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
            Ok(3.15)
        }

        fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
            Ok(2.72)
        }

        fn read_big_integer(
            &mut self,
            _schema: &Schema<'_>,
        ) -> Result<aws_smithy_types::BigInteger, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigInteger::from_str("12345").unwrap())
        }

        fn read_big_decimal(
            &mut self,
            _schema: &Schema<'_>,
        ) -> Result<aws_smithy_types::BigDecimal, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigDecimal::from_str("123.45").unwrap())
        }

        fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
            if self.index < self.values.len() {
                let value = self.values[self.index].clone();
                self.index += 1;
                Ok(value)
            } else {
                Ok("default".to_string())
            }
        }

        fn read_blob(
            &mut self,
            _schema: &Schema<'_>,
        ) -> Result<aws_smithy_types::Blob, SerdeError> {
            Ok(aws_smithy_types::Blob::new(vec![1, 2, 3, 4]))
        }

        fn read_timestamp(
            &mut self,
            _schema: &Schema<'_>,
        ) -> Result<aws_smithy_types::DateTime, SerdeError> {
            Ok(aws_smithy_types::DateTime::from_secs(1234567890))
        }

        fn read_document(
            &mut self,
            _schema: &Schema<'_>,
        ) -> Result<aws_smithy_types::Document, SerdeError> {
            Ok(aws_smithy_types::Document::Null)
        }

        fn is_null(&self) -> bool {
            false
        }

        fn container_size(&self) -> Option<usize> {
            Some(10)
        }
    }

    #[test]
    fn test_serializer_simple_types() {
        let mut ser = MockSerializer { output: Vec::new() };

        ser.write_boolean(&BOOLEAN, true).unwrap();
        ser.write_integer(&INTEGER, 42).unwrap();
        ser.write_string(&STRING, "hello").unwrap();

        let output = ser.finish();
        assert_eq!(output, vec!["bool(true)", "int(42)", "string(hello)"]);
    }

    #[test]
    fn test_serializer_struct() {
        // A simple struct that serializes two fields
        struct TestStruct;
        impl SerializableStruct for TestStruct {
            fn serialize_members(
                &self,
                serializer: &mut dyn ShapeSerializer,
            ) -> Result<(), SerdeError> {
                serializer.write_string(&STRING, "field1")?;
                serializer.write_integer(&INTEGER, 123)?;
                Ok(())
            }
        }

        let mut ser = MockSerializer { output: Vec::new() };
        ser.write_struct(&STRING, &TestStruct).unwrap();

        let output = ser.finish();
        assert_eq!(
            output,
            vec![
                "struct(smithy.api#String)",
                "string(field1)",
                "int(123)",
                "end_struct"
            ]
        );
    }

    #[test]
    fn test_deserializer_simple_types() {
        let mut deser = MockDeserializer::new(vec!["test".to_string()]);

        assert!(deser.read_boolean(&BOOLEAN).unwrap());
        assert_eq!(deser.read_integer(&INTEGER).unwrap(), 123456);
        assert_eq!(deser.read_string(&STRING).unwrap(), "test");
        assert_eq!(deser.container_size(), Some(10));
        assert!(!deser.is_null());
    }

    #[test]
    fn test_deserializer_struct() {
        let mut deser = MockDeserializer::new(vec!["value1".to_string(), "value2".to_string()]);

        let mut fields = Vec::new();
        deser
            .read_struct(&STRING, &mut |_member, d| {
                fields.push(d.read_string(&STRING)?);
                Ok(())
            })
            .unwrap();

        assert_eq!(fields, vec!["value1", "value2"]);
    }

    #[test]
    fn test_deserializer_list() {
        let mut deser =
            MockDeserializer::new(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        let mut elements = Vec::new();
        deser
            .read_list(&STRING, &mut |d| {
                elements.push(d.read_string(&STRING)?);
                Ok(())
            })
            .unwrap();

        assert_eq!(elements, vec!["a", "b", "c"]);
    }

    // Records the shape id of every schema handed to an element/value read,
    // so the default collection helpers can be checked to pass the *member*
    // (list element / map value) schema — the carrier of element-level traits
    // such as `@mediaType` / `@timestampFormat` — rather than the container
    // schema.
    #[derive(Default)]
    struct SchemaRecordingDeserializer {
        seen: Vec<String>,
    }

    impl ShapeDeserializer for SchemaRecordingDeserializer {
        fn read_struct(
            &mut self,
            _schema: &Schema<'_>,
            _consumer: &mut dyn FnMut(
                &Schema<'_>,
                &mut dyn ShapeDeserializer,
            ) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn read_list(
            &mut self,
            _schema: &Schema<'_>,
            consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            consumer(self)
        }

        fn read_map(
            &mut self,
            _schema: &Schema<'_>,
            consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            consumer("k".to_string(), self)
        }

        fn read_boolean(&mut self, _s: &Schema<'_>) -> Result<bool, SerdeError> {
            Ok(false)
        }
        fn read_byte(&mut self, _s: &Schema<'_>) -> Result<i8, SerdeError> {
            Ok(0)
        }
        fn read_short(&mut self, _s: &Schema<'_>) -> Result<i16, SerdeError> {
            Ok(0)
        }
        fn read_integer(&mut self, schema: &Schema<'_>) -> Result<i32, SerdeError> {
            self.seen.push(schema.shape_id().as_str().to_string());
            Ok(0)
        }
        fn read_long(&mut self, schema: &Schema<'_>) -> Result<i64, SerdeError> {
            self.seen.push(schema.shape_id().as_str().to_string());
            Ok(0)
        }
        fn read_float(&mut self, _s: &Schema<'_>) -> Result<f32, SerdeError> {
            Ok(0.0)
        }
        fn read_double(&mut self, _s: &Schema<'_>) -> Result<f64, SerdeError> {
            Ok(0.0)
        }
        fn read_big_integer(
            &mut self,
            _s: &Schema<'_>,
        ) -> Result<aws_smithy_types::BigInteger, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigInteger::from_str("0").unwrap())
        }
        fn read_big_decimal(
            &mut self,
            _s: &Schema<'_>,
        ) -> Result<aws_smithy_types::BigDecimal, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigDecimal::from_str("0").unwrap())
        }
        fn read_string(&mut self, schema: &Schema<'_>) -> Result<String, SerdeError> {
            self.seen.push(schema.shape_id().as_str().to_string());
            Ok(String::new())
        }
        fn read_blob(&mut self, schema: &Schema<'_>) -> Result<aws_smithy_types::Blob, SerdeError> {
            self.seen.push(schema.shape_id().as_str().to_string());
            Ok(aws_smithy_types::Blob::new(Vec::new()))
        }
        fn read_timestamp(
            &mut self,
            _s: &Schema<'_>,
        ) -> Result<aws_smithy_types::DateTime, SerdeError> {
            Ok(aws_smithy_types::DateTime::from_secs(0))
        }
        fn read_document(
            &mut self,
            _s: &Schema<'_>,
        ) -> Result<aws_smithy_types::Document, SerdeError> {
            Ok(aws_smithy_types::Document::Null)
        }
        fn is_null(&self) -> bool {
            false
        }
        fn container_size(&self) -> Option<usize> {
            None
        }
    }

    #[test]
    fn default_string_list_passes_member_schema_to_elements() {
        let member = Schema::new(
            crate::shape_id!("test", "ListMember"),
            crate::ShapeType::String,
        );
        let list = Schema::new_list(crate::shape_id!("test", "StringList"), &member);

        let mut deser = SchemaRecordingDeserializer::default();
        deser.read_string_list(&list).unwrap();
        // The element read sees the member schema, not the container list.
        assert_eq!(deser.seen, vec!["test#ListMember".to_string()]);
    }

    #[test]
    fn default_integer_list_passes_member_schema_to_elements() {
        let member = Schema::new(
            crate::shape_id!("test", "IntMember"),
            crate::ShapeType::Integer,
        );
        let list = Schema::new_list(crate::shape_id!("test", "IntList"), &member);

        let mut deser = SchemaRecordingDeserializer::default();
        deser.read_integer_list(&list).unwrap();
        assert_eq!(deser.seen, vec!["test#IntMember".to_string()]);
    }

    #[test]
    fn default_string_map_passes_value_schema_to_values() {
        let key = Schema::new(crate::shape_id!("test", "MapKey"), crate::ShapeType::String);
        let value = Schema::new(
            crate::shape_id!("test", "MapValue"),
            crate::ShapeType::String,
        );
        let map = Schema::new_map(crate::shape_id!("test", "StringMap"), &key, &value);

        let mut deser = SchemaRecordingDeserializer::default();
        deser.read_string_string_map(&map).unwrap();
        // The value read sees the map value schema, not the container map.
        assert_eq!(deser.seen, vec!["test#MapValue".to_string()]);
    }

    #[test]
    fn default_collection_helper_falls_back_to_prelude_without_member() {
        // A list schema built without a member set falls back to the prelude
        // scalar rather than (incorrectly) passing the container schema.
        let bare_list = Schema::new(crate::shape_id!("test", "BareList"), crate::ShapeType::List);

        let mut deser = SchemaRecordingDeserializer::default();
        deser.read_string_list(&bare_list).unwrap();
        assert_eq!(deser.seen, vec!["smithy.api#String".to_string()]);
    }

    #[test]
    fn test_deserializer_map() {
        let mut deser = MockDeserializer::new(vec!["val1".to_string(), "val2".to_string()]);

        let mut entries = Vec::new();
        deser
            .read_map(&STRING, &mut |key, d| {
                let value = d.read_string(&STRING)?;
                entries.push((key, value));
                Ok(())
            })
            .unwrap();

        assert_eq!(
            entries,
            vec![
                ("key1".to_string(), "val1".to_string()),
                ("key2".to_string(), "val2".to_string())
            ]
        );
    }
}

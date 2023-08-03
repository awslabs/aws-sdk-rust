/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Binary Blob Type
///
/// Blobs represent protocol-agnostic binary content.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Blob {
    inner: Vec<u8>,
}

impl Blob {
    /// Creates a new blob from the given `input`.
    pub fn new<T: Into<Vec<u8>>>(input: T) -> Self {
        Blob {
            inner: input.into(),
        }
    }

    /// Consumes the `Blob` and returns a `Vec<u8>` with its contents.
    pub fn into_inner(self) -> Vec<u8> {
        self.inner
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

#[cfg(all(aws_sdk_unstable, feature = "serde-serialize"))]
mod serde_serialize {
    use super::*;
    use crate::base64;
    use serde::Serialize;

    impl Serialize for Blob {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            if serializer.is_human_readable() {
                serializer.serialize_str(&crate::base64::encode(&self.inner))
            } else {
                serializer.serialize_bytes(&self.inner)
            }
        }
    }
}

#[cfg(all(aws_sdk_unstable, feature = "serde-deserialize"))]
mod serde_deserialize {
    use super::*;
    use crate::base64;
    use serde::{de::Visitor, Deserialize};

    struct HumanReadableBlobVisitor;
    impl<'de> Visitor<'de> for HumanReadableBlobVisitor {
        type Value = Blob;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("expected base64 encoded string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match base64::decode(v) {
                Ok(inner) => Ok(Blob { inner }),
                Err(e) => Err(E::custom(e)),
            }
        }
    }

    struct NotHumanReadableBlobVisitor;
    impl<'de> Visitor<'de> for NotHumanReadableBlobVisitor {
        type Value = Blob;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("expected bytes")
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Blob { inner: v })
        }
    }

    impl<'de> Deserialize<'de> for Blob {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            if deserializer.is_human_readable() {
                deserializer.deserialize_str(HumanReadableBlobVisitor)
            } else {
                deserializer.deserialize_byte_buf(NotHumanReadableBlobVisitor)
            }
        }
    }
}

#[cfg(test)]
#[cfg(all(
    aws_sdk_unstable,
    feature = "serde-serialize",
    feature = "serde-deserialize"
))]
mod test_serde {
    use crate::Blob;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct ForTest {
        blob: Blob,
    }

    #[test]
    fn human_readable_blob() {
        let aws_in_base64 = r#"{"blob":"QVdT"}"#;
        let for_test = ForTest {
            blob: Blob {
                inner: vec![b'A', b'W', b'S'],
            },
        };
        assert_eq!(for_test, serde_json::from_str(aws_in_base64).unwrap());
        assert_eq!(serde_json::to_string(&for_test).unwrap(), aws_in_base64);
    }

    #[test]
    fn not_human_readable_blob() {
        use std::ffi::CString;

        let for_test = ForTest {
            blob: Blob {
                inner: vec![b'A', b'W', b'S'],
            },
        };
        let mut buf = vec![];
        let res = ciborium::ser::into_writer(&for_test, &mut buf);
        assert!(res.is_ok());

        // checks whether the bytes are deserialized properly
        let n: HashMap<String, CString> =
            ciborium::de::from_reader(std::io::Cursor::new(buf.clone())).unwrap();
        assert!(n.get("blob").is_some());
        assert!(n.get("blob") == CString::new([65, 87, 83]).ok().as_ref());

        let de: ForTest = ciborium::de::from_reader(std::io::Cursor::new(buf)).unwrap();
        assert_eq!(for_test, de);
    }
}

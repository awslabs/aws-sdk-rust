/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use bytes::Bytes;
use std::any::Any;
use std::fmt;

/// Binary Blob Type
///
/// Blobs represent protocol-agnostic binary content.
///
/// The [`fmt::Debug`] and [`fmt::Display`] implementations render the contents
/// as a lowercase hex-encoded string. This avoids noisy byte-array output
/// in service logs while still preserving the underlying data.
#[derive(Default, PartialEq, Eq, Hash, Clone)]
pub struct Blob {
    inner: Bytes,
}

struct B<'b>(&'b [u8]);

impl std::fmt::Debug for B<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|b| write!(f, "{b:02x}"))
    }
}

impl fmt::Debug for Blob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Blob")
            .field("inner", &format_args!("{:?}", B(&self.inner)))
            .finish()
    }
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", B(&self.inner))
    }
}

impl Blob {
    /// Creates a new blob from the given `input`.
    pub fn new<T: Into<Vec<u8>>>(input: T) -> Self {
        Blob {
            inner: input.into().into(),
        }
    }

    /// Creates a new blob, reusing an existing shared allocation when possible.
    pub fn from_maybe_shared<T: AsRef<[u8]> + 'static>(input: T) -> Self {
        let mut input = Some(input);
        let inner =
            if let Some(bytes) = (&mut input as &mut dyn Any).downcast_mut::<Option<Bytes>>() {
                bytes.take().expect("input is present")
            } else {
                Bytes::copy_from_slice(input.as_ref().expect("input is present").as_ref())
            };
        Blob { inner }
    }

    /// Consumes the `Blob` and returns its contents as `Bytes`.
    pub fn into_bytes(self) -> Bytes {
        self.inner
    }

    /// Consumes the `Blob` and returns a `Vec<u8>` with its contents.
    pub fn into_inner(self) -> Vec<u8> {
        self.inner.into()
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl From<Vec<u8>> for Blob {
    fn from(value: Vec<u8>) -> Self {
        Blob::new(value)
    }
}

impl From<Blob> for Vec<u8> {
    fn from(value: Blob) -> Self {
        value.into_inner()
    }
}

impl From<&[u8]> for Blob {
    fn from(value: &[u8]) -> Self {
        Blob::new(value)
    }
}

#[cfg(all(aws_sdk_unstable, feature = "serde-serialize"))]
mod serde_serialize {
    use super::*;
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
            match crate::base64::decode(v) {
                Ok(inner) => Ok(Blob::from(inner)),
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
            Ok(Blob::from(v))
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
mod test {
    use crate::Blob;
    use bytes::Bytes;

    #[test]
    fn blob_conversion() {
        let my_bytes: &[u8] = &[1u8, 2u8, 3u8];
        let my_vec = vec![1u8, 2u8, 3u8];
        let orig_vec = my_vec.clone();

        let blob1: Blob = my_bytes.into();
        let vec1: Vec<u8> = blob1.into();
        assert_eq!(orig_vec, vec1);

        let blob2: Blob = my_vec.into();
        let vec2: Vec<u8> = blob2.into();
        assert_eq!(orig_vec, vec2);
    }

    #[test]
    fn blob_reuses_bytes() {
        let bytes = Bytes::from_static(b"some shared bytes");
        let original_ptr = bytes.as_ptr();

        let blob = Blob::from_maybe_shared(bytes);
        let bytes = blob.into_bytes();

        assert_eq!(original_ptr, bytes.as_ptr());
    }

    #[test]
    fn blob_new_accepts_borrowed_data() {
        let bytes = vec![1, 2, 3];
        let blob = Blob::new(bytes.as_slice());

        assert_eq!(bytes, blob.as_ref());
    }

    #[test]
    fn blob_display_is_hex_encoded() {
        let blob = Blob::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(format!("{blob}"), "deadbeef");
    }

    #[test]
    fn blob_debug_is_hex_encoded() {
        let blob = Blob::new(vec![0x00, 0x01, 0x02, 0xFF]);
        assert_eq!(format!("{blob:?}"), "Blob { inner: 000102ff }");
    }

    #[test]
    fn empty_blob_display_and_debug() {
        let blob = Blob::new(Vec::<u8>::new());
        assert_eq!(format!("{blob}"), "");
        assert_eq!(format!("{blob:?}"), "Blob { inner:  }");
    }
}

#[cfg(all(
    aws_sdk_unstable,
    feature = "serde-serialize",
    feature = "serde-deserialize"
))]
mod test_serde {
    use crate::Blob;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    #[allow(dead_code)]
    struct ForTest {
        blob: Blob,
    }

    #[test]
    fn human_readable_blob() {
        let aws_in_base64 = r#"{"blob":"QVdT"}"#;
        let for_test = ForTest {
            blob: Blob {
                inner: vec![b'A', b'W', b'S'].into(),
            },
        };
        assert_eq!(for_test, serde_json::from_str(aws_in_base64).unwrap());
        assert_eq!(serde_json::to_string(&for_test).unwrap(), aws_in_base64);
    }

    #[test]
    fn not_human_readable_blob() {
        use std::collections::HashMap;
        use std::ffi::CString;

        let for_test = ForTest {
            blob: Blob {
                inner: vec![b'A', b'W', b'S'].into(),
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

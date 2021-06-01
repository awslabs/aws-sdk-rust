/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::borrow::Cow;

/// Escapes a string for embedding in a JSON string value.
pub fn escape_string(value: &str) -> Cow<str> {
    let bytes = value.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        match byte {
            0..=0x1F | b'"' | b'\\' => {
                return Cow::Owned(escape_string_inner(&bytes[0..index], &bytes[index..]))
            }
            _ => {}
        }
    }
    Cow::Borrowed(value)
}

fn escape_string_inner(start: &[u8], rest: &[u8]) -> String {
    let mut escaped = Vec::with_capacity(start.len() + rest.len() + 1);
    escaped.extend(start);

    for byte in rest {
        match byte {
            b'"' => escaped.extend(b"\\\""),
            b'\\' => escaped.extend(b"\\\\"),
            0x08 => escaped.extend(b"\\b"),
            0x0C => escaped.extend(b"\\f"),
            b'\n' => escaped.extend(b"\\n"),
            b'\r' => escaped.extend(b"\\r"),
            b'\t' => escaped.extend(b"\\t"),
            0..=0x1F => escaped.extend(format!("\\u{:04x}", byte).bytes()),
            _ => escaped.push(*byte),
        }
    }

    // This is safe because:
    // - The original input was valid UTF-8 since it came in as a `&str`
    // - Only single-byte code points were escaped
    // - The escape sequences are valid UTF-8
    debug_assert!(std::str::from_utf8(&escaped).is_ok());
    unsafe { String::from_utf8_unchecked(escaped) }
}

#[cfg(test)]
mod test {
    use super::escape_string;

    #[test]
    fn escape() {
        assert_eq!("", escape_string("").as_ref());
        assert_eq!("foo", escape_string("foo").as_ref());
        assert_eq!("foo\\r\\n", escape_string("foo\r\n").as_ref());
        assert_eq!("foo\\r\\nbar", escape_string("foo\r\nbar").as_ref());
        assert_eq!(r#"foo\\bar"#, escape_string(r#"foo\bar"#).as_ref());
        assert_eq!(r#"\\foobar"#, escape_string(r#"\foobar"#).as_ref());
        assert_eq!(
            r#"\bf\fo\to\r\n"#,
            escape_string("\u{08}f\u{0C}o\to\r\n").as_ref()
        );
        assert_eq!("\\\"test\\\"", escape_string("\"test\"").as_ref());
        assert_eq!("\\u0000", escape_string("\u{0}").as_ref());
        assert_eq!("\\u001f", escape_string("\u{1f}").as_ref());
    }

    use proptest::proptest;
    proptest! {
        #[test]
        fn matches_serde_json(s in ".*") {
            let serde_escaped = serde_json::to_string(&s).unwrap();
            let serde_escaped = &serde_escaped[1..(serde_escaped.len() - 1)];
            assert_eq!(serde_escaped,escape_string(&s))
        }
    }

    #[test]
    #[ignore] // This tests escaping of all codepoints, but can take a long time in debug builds
    fn all_codepoints() {
        for value in 0..u32::MAX {
            if let Some(chr) = char::from_u32(value) {
                let string = String::from(chr);
                let escaped = escape_string(&string);
                let serde_escaped = serde_json::to_string(&string).unwrap();
                let serde_escaped = &serde_escaped[1..(serde_escaped.len() - 1)];
                assert_eq!(&escaped, serde_escaped);
            }
        }
    }
}

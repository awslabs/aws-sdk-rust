/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Formatting values as Smithy
//! [httpLabel](https://awslabs.github.io/smithy/1.0/spec/core/http-traits.html#httplabel-trait)

use crate::urlencode::BASE_SET;
use aws_smithy_types::date_time::{DateTimeFormatError, Format};
use aws_smithy_types::DateTime;
use percent_encoding::AsciiSet;

const GREEDY: &AsciiSet = &BASE_SET.remove(b'/');

pub fn fmt_string<T: AsRef<str>>(t: T, greedy: bool) -> String {
    let uri_set = if greedy { GREEDY } else { BASE_SET };
    percent_encoding::utf8_percent_encode(t.as_ref(), uri_set).to_string()
}

pub fn fmt_timestamp(t: &DateTime, format: Format) -> Result<String, DateTimeFormatError> {
    Ok(crate::query::fmt_string(t.fmt(format)?))
}

#[cfg(test)]
mod test {
    use crate::label::fmt_string;
    use http::Uri;
    use proptest::proptest;

    #[test]
    fn greedy_params() {
        assert_eq!(fmt_string("a/b", false), "a%2Fb");
        assert_eq!(fmt_string("a/b", true), "a/b");
    }

    proptest! {
        #[test]
        fn test_encode_request(s: String) {
            let _: Uri = format!("http://host.example.com/{}", fmt_string(&s, false)).parse().expect("all strings should be encoded properly");
            let _: Uri = format!("http://host.example.com/{}", fmt_string(&s, true)).parse().expect("all strings should be encoded properly");
        }
    }
}

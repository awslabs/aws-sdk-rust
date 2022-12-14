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

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EncodingStrategy {
    Default,
    Greedy,
}

pub fn fmt_string<T: AsRef<str>>(t: T, strategy: EncodingStrategy) -> String {
    let uri_set = if strategy == EncodingStrategy::Greedy {
        GREEDY
    } else {
        BASE_SET
    };
    percent_encoding::utf8_percent_encode(t.as_ref(), uri_set).to_string()
}

pub fn fmt_timestamp(t: &DateTime, format: Format) -> Result<String, DateTimeFormatError> {
    Ok(fmt_string(t.fmt(format)?, EncodingStrategy::Default))
}

#[cfg(test)]
mod test {
    use crate::label::{fmt_string, EncodingStrategy};
    use http::Uri;
    use proptest::proptest;

    #[test]
    fn greedy_params() {
        assert_eq!(fmt_string("a/b", EncodingStrategy::Default), "a%2Fb");
        assert_eq!(fmt_string("a/b", EncodingStrategy::Greedy), "a/b");
    }

    proptest! {
        #[test]
        fn test_encode_request(s: String) {
            let _: Uri = format!("http://host.example.com/{}", fmt_string(&s, EncodingStrategy::Default))
                .parse()
                .expect("all strings should be encoded properly");
            let _: Uri = format!("http://host.example.com/{}", fmt_string(&s, EncodingStrategy::Greedy))
                .parse()
                .expect("all strings should be encoded properly");
        }
    }
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

/// Formatting values as Smithy
/// [httpLabel](https://awslabs.github.io/smithy/1.0/spec/core/http-traits.html#httplabel-trait)
use smithy_types::Instant;
use std::fmt::Debug;

pub fn fmt_default<T: Debug>(t: T) -> String {
    format!("{:?}", t)
}

pub fn fmt_string<T: AsRef<str>>(t: T, greedy: bool) -> String {
    let s = t.as_ref().replace(":", "%3A");
    if greedy {
        s
    } else {
        s.replace("/", "%2F")
    }
}

pub fn fmt_timestamp(t: &Instant, format: smithy_types::instant::Format) -> String {
    crate::query::fmt_string(t.fmt(format))
}

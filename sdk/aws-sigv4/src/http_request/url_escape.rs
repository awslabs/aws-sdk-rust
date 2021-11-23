/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use percent_encoding::{AsciiSet, CONTROLS};

/// base set of characters that must be URL encoded
const BASE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    // RFC-3986 §3.3 allows sub-delims (defined in section2.2) to be in the path component.
    // This includes both colon ':' and comma ',' characters.
    // Smithy protocol tests & AWS services percent encode these expected values. Signing
    // will fail if these values are not percent encoded
    .add(b':')
    .add(b',')
    .add(b'?')
    .add(b'#')
    .add(b'[')
    .add(b']')
    .add(b'@')
    .add(b'!')
    .add(b'$')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b';')
    .add(b'=')
    .add(b'%');

const QUERY_SET: &AsciiSet = &BASE_SET.add(b'/');
const PATH_SET: &AsciiSet = BASE_SET;

pub(super) fn percent_encode_query(value: &str) -> String {
    percent_encoding::percent_encode(value.as_bytes(), QUERY_SET).to_string()
}

pub(super) fn percent_encode_path(value: &str) -> String {
    percent_encoding::percent_encode(value.as_bytes(), PATH_SET).to_string()
}

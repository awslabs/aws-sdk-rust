/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// This fixture must FAIL to compile. `aws_smithy_types::Document` is
// `#[non_exhaustive]`, so a downstream crate cannot match it exhaustively
// without a wildcard (`_`) arm, even when it names every variant that exists
// today. That is what makes adding a future variant a non-breaking change.
//
// If `#[non_exhaustive]` is ever removed from `Document`, this file will start
// compiling and the `document_match_without_wildcard_arm_fails_to_compile`
// trybuild test will fail, flagging the regression.

use aws_smithy_types::Document;

fn label(d: &Document) -> &'static str {
    match d {
        Document::Object(_) => "object",
        Document::Array(_) => "array",
        Document::Number(_) => "number",
        Document::String(_) => "string",
        Document::Bool(_) => "bool",
        Document::Null => "null",
        Document::Blob(_) => "blob",
        Document::Timestamp(_) => "timestamp",
        Document::BigInteger(_) => "bigInteger",
        Document::BigDecimal(_) => "bigDecimal",
    }
}

fn main() {
    let _ = label(&Document::Null);
}

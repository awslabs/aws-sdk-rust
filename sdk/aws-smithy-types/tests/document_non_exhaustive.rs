/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Compile-time contract tests for `aws_smithy_types::Document`.
//!
//! `Document` is `#[non_exhaustive]`. That attribute only takes effect for
//! *consuming* crates, so it cannot be exercised by a unit test inside
//! `aws-smithy-types` itself. These `trybuild` cases compile fixtures as
//! separate downstream crates, where the attribute is in force.

/// Asserts that a downstream exhaustive `match` over `Document` that omits the
/// wildcard (`_`) arm fails to compile, which is the observable consequence of
/// `Document` being `#[non_exhaustive]`.
#[test]
fn document_match_without_wildcard_arm_fails_to_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/document_match_missing_wildcard.rs");
}

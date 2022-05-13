/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::{label, query};

pub(super) fn percent_encode_query(value: &str) -> String {
    query::fmt_string(value)
}

pub(super) fn percent_encode_path(value: &str) -> String {
    label::fmt_string(value, true)
}

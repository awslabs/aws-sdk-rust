/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP specific retry behaviors
//!
//! For protocol agnostic retries, see `aws_smithy_types::Retry`.

use aws_smithy_types::retry::RetryKind;

pub trait ClassifyResponse<T, E>: Clone {
    fn classify(&self, response: Result<&T, &E>) -> RetryKind;
}

impl<T, E> ClassifyResponse<T, E> for () {
    fn classify(&self, _: Result<&T, &E>) -> RetryKind {
        RetryKind::Unnecessary
    }
}

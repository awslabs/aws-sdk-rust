/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Use aws_runtime::content_encoding::header_value instead.
#[deprecated(
    since = "0.60.2",
    note = "Use aws_runtime::content_encoding::header_value instead."
)]
pub mod header_value {
    /// Use aws_runtime::content_encoding::header_value::AWS_CHUNKED instead.
    #[deprecated(
        since = "0.60.2",
        note = "Use aws_runtime::content_encoding::header_value::AWS_CHUNKED instead."
    )]
    pub const AWS_CHUNKED: &str = "aws-chunked";
}

/// Use aws_runtime::content_encoding::AwsChunkedBodyOption instead.
#[deprecated(
    since = "0.60.2",
    note = "Use aws_runtime::content_encoding::AwsChunkedBodyOptions instead."
)]
pub type AwsChunkedBodyOptions = aws_runtime::content_encoding::AwsChunkedBodyOptions;

/// Use aws_runtime::content_encoding::AwsChunkedBody instead.
#[deprecated(
    since = "0.60.2",
    note = "Use aws_runtime::content_encoding::AwsChunkedBody instead."
)]
pub type AwsChunkedBody<Inner> = aws_runtime::content_encoding::AwsChunkedBody<Inner>;

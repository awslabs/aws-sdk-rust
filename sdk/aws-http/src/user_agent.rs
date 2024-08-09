/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Use aws_runtime::user_agent::AwsUserAgent instead.
#[deprecated(
    since = "0.60.2",
    note = "Use aws_runtime::user_agent::AwsUserAgent instead."
)]
pub type AwsUserAgent = aws_runtime::user_agent::AwsUserAgent;

/// Use aws_runtime::user_agent::ApiMetadata instead.
#[deprecated(
    since = "0.60.2",
    note = "Use aws_runtime::user_agent::ApiMetadata instead."
)]
pub type ApiMetadata = aws_runtime::user_agent::ApiMetadata;

/// Use aws_runtime::user_agent::InvalidMetadataValue instead.
#[deprecated(
    since = "0.60.2",
    note = "Use aws_runtime::user_agent::InvalidMetadataValue instead."
)]
pub type InvalidMetadataValue = aws_runtime::user_agent::InvalidMetadataValue;

/// Use aws_runtime::user_agent::AdditionalMetadata instead.
#[deprecated(
    since = "0.60.2",
    note = "Use aws_runtime::user_agent::AdditionalMetadata instead."
)]
pub type AdditionalMetadata = aws_runtime::user_agent::AdditionalMetadata;

/// Use aws_runtime::user_agent::BusinessMetric instead.
#[deprecated(
    since = "0.60.2",
    note = "Use aws_runtime::user_agent::BusinessMetric instead."
)]
#[allow(deprecated)]
pub type FeatureMetadata = aws_runtime::user_agent::FeatureMetadata;

/// Use aws_runtime::user_agent::BusinessMetric instead.
#[deprecated(
    since = "0.60.2",
    note = "Use aws_runtime::user_agent::BusinessMetric instead."
)]
#[allow(deprecated)]
pub type ConfigMetadata = aws_runtime::user_agent::ConfigMetadata;

/// Use aws_runtime::user_agent::FrameworkMetadata instead.
#[deprecated(
    since = "0.60.2",
    note = "Use aws_runtime::user_agent::FrameworkMetadata instead."
)]
pub type FrameworkMetadata = aws_runtime::user_agent::FrameworkMetadata;

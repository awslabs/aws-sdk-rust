/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//TODO(runtimeCratesVersioningCleanup): Keep the following deprecated type aliases for at least
// one release since 0.56.1 and then remove this module.

//! Types for [`error`](aws_smithy_runtime_api::client::result::SdkError) responses.

/// Builders for `SdkError` variant context.
pub mod builders {
    /// Builder for [`ConstructionFailure`](aws_smithy_runtime_api::client::result::ConstructionFailure).
    #[deprecated(
        note = "Moved to `aws_smithy_runtime_api::client::result::builders::ConstructionFailureBuilder`."
    )]
    pub type ConstructionFailureBuilder =
        aws_smithy_runtime_api::client::result::builders::ConstructionFailureBuilder;

    /// Builder for [`TimeoutError`](aws_smithy_runtime_api::client::result::TimeoutError).
    #[deprecated(
        note = "Moved to `aws_smithy_runtime_api::client::result::builders::TimeoutErrorBuilder`."
    )]
    pub type TimeoutErrorBuilder =
        aws_smithy_runtime_api::client::result::builders::TimeoutErrorBuilder;

    /// Builder for [`DispatchFailure`](aws_smithy_runtime_api::client::result::DispatchFailure).
    #[deprecated(
        note = "Moved to `aws_smithy_runtime_api::client::result::builders::DispatchFailureBuilder`."
    )]
    pub type DispatchFailureBuilder =
        aws_smithy_runtime_api::client::result::builders::DispatchFailureBuilder;

    /// Builder for [`ResponseError`](aws_smithy_runtime_api::client::result::ResponseError).
    #[deprecated(
        note = "Moved to `aws_smithy_runtime_api::client::result::builders::ResponseErrorBuilder`."
    )]
    pub type ResponseErrorBuilder<R> =
        aws_smithy_runtime_api::client::result::builders::ResponseErrorBuilder<R>;

    /// Builder for [`ServiceError`](aws_smithy_runtime_api::client::result::ServiceError).
    #[deprecated(
        note = "Moved to `aws_smithy_runtime_api::client::result::builders::ServiceErrorBuilder`."
    )]
    pub type ServiceErrorBuilder<E, R> =
        aws_smithy_runtime_api::client::result::builders::ServiceErrorBuilder<E, R>;
}

/// Error context for [`aws_smithy_runtime_api::client::result::ConstructionFailure`]
#[deprecated(note = "Moved to `aws_smithy_runtime_api::client::result::ConstructionFailure`.")]
pub type ConstructionFailure = aws_smithy_runtime_api::client::result::ConstructionFailure;

/// Error context for [`aws_smithy_runtime_api::client::result::TimeoutError`]
#[deprecated(note = "Moved to `aws_smithy_runtime_api::client::result::TimeoutError`.")]
pub type TimeoutError = aws_smithy_runtime_api::client::result::TimeoutError;

/// Error context for [`aws_smithy_runtime_api::client::result::DispatchFailure`]
#[deprecated(note = "Moved to `aws_smithy_runtime_api::client::result::DispatchFailure`.")]
pub type DispatchFailure = aws_smithy_runtime_api::client::result::DispatchFailure;

/// Error context for [`aws_smithy_runtime_api::client::result::ResponseError`]
#[deprecated(note = "Moved to `aws_smithy_runtime_api::client::result::ResponseError`.")]
pub type ResponseError<R> = aws_smithy_runtime_api::client::result::ResponseError<R>;

/// Failed SDK Result
#[deprecated(note = "Moved to `aws_smithy_runtime_api::client::result::ServiceError`.")]
pub type ServiceError<E, R> = aws_smithy_runtime_api::client::result::ServiceError<E, R>;

/// Failed SDK Result
#[deprecated(note = "Moved to `aws_smithy_runtime_api::client::result::SdkError`.")]
pub type SdkError<E, R> = aws_smithy_runtime_api::client::result::SdkError<E, R>;

/// Error from the underlying Connector
#[deprecated(note = "Moved to `aws_smithy_runtime_api::client::result::ConnectorError`.")]
pub type ConnectorError = aws_smithy_runtime_api::client::result::ConnectorError;

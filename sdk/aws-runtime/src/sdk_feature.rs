/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Note: This code originally lived in the `aws-runtime` crate. It was moved here to avoid circular dependencies
/// This module is re-exported in `aws-runtime`, and so even though this is a pre-1.0 crate, this module should not
/// have any breaking changes
use aws_smithy_types::config_bag::{Storable, StoreAppend};

/// IDs for the features that may be used in the AWS SDK
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AwsSdkFeature {
    /// An operation called with account ID mode set to preferred
    AccountIdModePreferred,
    /// An operation called with account ID mode set to disabled
    AccountIdModeDisabled,
    /// An operation called with account ID mode set to required
    AccountIdModeRequired,
    /// Indicates that an operation was called by the S3 Transfer Manager
    S3Transfer,
    /// Calling an SSO-OIDC operation as part of the SSO login flow, when using the OAuth2.0 device code grant
    SsoLoginDevice,
    /// Calling an SSO-OIDC operation as part of the SSO login flow, when using the OAuth2.0 authorization code grant
    SsoLoginAuth,
    /// Indicates that a custom endpoint URL was configured
    EndpointOverride,
    /// Indicates that an upload directory operation was called by the S3 Transfer Manager
    S3TransferUploadDirectory,
    /// Indicates that a download directory operation was called by the S3 Transfer Manager
    S3TransferDownloadDirectory,
    /// Indicates that the S3 client was configured with a non-default part size
    S3CustomPartSize,
    /// Indicates that the S3 client was configured with a non-default throughput target
    S3CustomThroughput,
    /// Indicates that the S3 client was configured with a non-default memory pool size
    S3CustomMemoryLimit,
    /// Indicates that the S3 client detected it is running on an EC2 instance
    S3OnEc2,
    /// Indicates that an operation used a file path rather than a streaming body
    S3FilePath,
}

impl Storable for AwsSdkFeature {
    type Storer = StoreAppend<Self>;
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_sts as sts;
use aws_smithy_types::error::Error as ErrorMeta;
use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind};
use sts::error::{
    AssumeRoleWithWebIdentityError, AssumeRoleWithWebIdentityErrorKind,
    IdpCommunicationErrorException,
};

#[tokio::test]
async fn idp_comms_err_retryable() {
    let error = AssumeRoleWithWebIdentityError::new(
        AssumeRoleWithWebIdentityErrorKind::IdpCommunicationErrorException(
            IdpCommunicationErrorException::builder()
                .message("test")
                .build(),
        ),
        ErrorMeta::builder()
            .code("IDPCommunicationError")
            .message("test")
            .build(),
    );
    assert_eq!(
        Some(ErrorKind::ServerError),
        error.retryable_error_kind(),
        "IdpCommunicationErrorException should be a retryable server error"
    );
}

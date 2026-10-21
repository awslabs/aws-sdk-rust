/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_runtime::retries::classifiers::AwsErrorCodeClassifier;
use aws_sdk_kms as kms;
use aws_smithy_runtime_api::client::interceptors::context::{Error, Input, InterceptorContext};
use aws_smithy_runtime_api::client::orchestrator::OrchestratorError;
use aws_smithy_runtime_api::client::retries::classifiers::{ClassifyRetry, RetryAction};
use aws_smithy_types::error::ErrorMetadata;
use kms::operation::create_alias::CreateAliasError;

// Classify an error carrying `error_code`, the way a protocol's error parser populates it.
//
// `CreateAliasError::generic` takes the metadata directly, so this test holds no wire bytes.
fn classify(error_code: &str) -> RetryAction {
    let err = CreateAliasError::generic(ErrorMetadata::builder().code(error_code).build());
    let mut ctx = InterceptorContext::new(Input::doesnt_matter());
    ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(err))));
    AwsErrorCodeClassifier::<CreateAliasError>::new().classify_retry(&ctx)
}

#[test]
fn errors_are_retryable() {
    assert_eq!(
        RetryAction::throttling_error(),
        classify("LimitExceededException")
    );
}

#[test]
fn unmodeled_errors_are_retryable() {
    assert_eq!(
        RetryAction::throttling_error(),
        classify("ThrottlingException")
    );
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::operation::get_object::{GetObjectError, GetObjectOutput};
use aws_smithy_runtime::{BoxError, ConfigBag, RetryStrategy};

//     retry_classifier: Arc::new(
//         |res: Result<&SdkSuccess<GetObjectOutput>, &SdkError<GetObjectError>>| -> RetryKind {
//             let classifier = AwsResponseRetryClassifier::new();
//             classifier.classify_retry(res)
//         },
//     ),

#[derive(Debug)]
pub struct GetObjectRetryStrategy {}

impl GetObjectRetryStrategy {
    pub fn _new() -> Self {
        Self {}
    }
}

impl RetryStrategy<Result<GetObjectOutput, GetObjectError>> for GetObjectRetryStrategy {
    fn should_retry(
        &self,
        _res: &Result<GetObjectOutput, GetObjectError>,
        _cfg: &ConfigBag,
    ) -> Result<bool, BoxError> {
        todo!()
    }
}

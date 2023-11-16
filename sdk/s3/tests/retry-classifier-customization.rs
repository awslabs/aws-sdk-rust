/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::config::interceptors::InterceptorContext;
use aws_sdk_s3::config::retry::{ClassifyRetry, RetryAction, RetryConfig};
use aws_sdk_s3::config::SharedAsyncSleep;
use aws_smithy_async::rt::sleep::TokioSleep;
use aws_smithy_runtime::client::http::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::body::SdkBody;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct CustomizationTestClassifier {
    counter: Arc<Mutex<u8>>,
}

impl CustomizationTestClassifier {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0u8)),
        }
    }

    pub fn counter(&self) -> u8 {
        *self.counter.lock().unwrap()
    }
}

impl ClassifyRetry for CustomizationTestClassifier {
    fn classify_retry(&self, ctx: &InterceptorContext) -> RetryAction {
        *self.counter.lock().unwrap() += 1;

        // Interceptors may call this classifier before a response is received. If a response was received,
        // ensure that it has the expected status code.
        if let Some(res) = ctx.response() {
            assert_eq!(
                500,
                res.status().as_u16(),
                "expected a 500 response from test connection"
            );
        }

        RetryAction::RetryForbidden
    }

    fn name(&self) -> &'static str {
        "Custom Retry Classifier"
    }
}

fn req() -> http::Request<SdkBody> {
    http::Request::builder()
        .body(SdkBody::from("request body"))
        .unwrap()
}

fn ok() -> http::Response<SdkBody> {
    http::Response::builder()
        .status(200)
        .body(SdkBody::from("Hello!"))
        .unwrap()
}

fn err() -> http::Response<SdkBody> {
    http::Response::builder()
        .status(500)
        .body(SdkBody::from("This was an error"))
        .unwrap()
}

#[tokio::test]
async fn test_retry_classifier_customization_for_service() {
    let http_client = StaticReplayClient::new(vec![
        ReplayEvent::new(req(), err()),
        ReplayEvent::new(req(), ok()),
    ]);

    let customization_test_classifier = CustomizationTestClassifier::new();

    let config = aws_sdk_s3::Config::builder()
        .with_test_defaults()
        .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
        .http_client(http_client)
        .retry_config(RetryConfig::standard())
        .retry_classifier(customization_test_classifier.clone())
        .build();

    let client = aws_sdk_s3::Client::from_conf(config);
    let _ = client
        .get_object()
        .bucket("bucket")
        .key("key")
        .send()
        .await
        .expect_err("fails without attempting a retry");

    // ensure our custom retry classifier was called at least once.
    assert_ne!(customization_test_classifier.counter(), 0);
}

#[tokio::test]
async fn test_retry_classifier_customization_for_operation() {
    let http_client = StaticReplayClient::new(vec![
        ReplayEvent::new(req(), err()),
        ReplayEvent::new(req(), ok()),
    ]);

    let customization_test_classifier = CustomizationTestClassifier::new();

    let config = aws_sdk_s3::Config::builder()
        .with_test_defaults()
        .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
        .http_client(http_client)
        .retry_config(RetryConfig::standard())
        .build();

    let client = aws_sdk_s3::Client::from_conf(config);
    let _ = client
        .get_object()
        .bucket("bucket")
        .key("key")
        .customize()
        .config_override(
            aws_sdk_s3::config::Config::builder()
                .retry_classifier(customization_test_classifier.clone()),
        )
        .send()
        .await
        .expect_err("fails without attempting a retry");

    // ensure our custom retry classifier was called at least once.
    assert_ne!(customization_test_classifier.counter(), 0);
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_endpoint::partition::endpoint::{Protocol, SignatureVersion};
use aws_endpoint::set_endpoint_resolver;
use aws_http::user_agent::AwsUserAgent;
use aws_http::AwsErrorRetryPolicy;
use aws_hyper::{Client, RetryConfig};
use aws_sig_auth::signer::OperationSigningConfig;
use aws_types::credentials::SharedCredentialsProvider;
use aws_types::region::Region;
use aws_types::Credentials;
use aws_types::SigningService;
use bytes::Bytes;
use http::header::{AUTHORIZATION, USER_AGENT};
use http::{self, Uri};
use smithy_client::test_connection::TestConnection;
use smithy_http::body::SdkBody;
use smithy_http::operation;
use smithy_http::operation::Operation;
use smithy_http::response::ParseHttpResponse;
use smithy_http::result::SdkError;
use smithy_types::retry::{ErrorKind, ProvideErrorKind};
use std::convert::Infallible;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use tokio::time::Instant;

#[derive(Clone)]
struct TestOperationParser;

#[derive(Debug)]
struct OperationError;

impl Display for OperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for OperationError {}

impl ProvideErrorKind for OperationError {
    fn retryable_error_kind(&self) -> Option<ErrorKind> {
        Some(ErrorKind::ThrottlingError)
    }

    fn code(&self) -> Option<&str> {
        None
    }
}

impl ParseHttpResponse for TestOperationParser {
    type Output = Result<String, OperationError>;

    fn parse_unloaded(&self, response: &mut operation::Response) -> Option<Self::Output> {
        if response.http().status().is_success() {
            Some(Ok("Hello!".to_string()))
        } else {
            Some(Err(OperationError))
        }
    }

    fn parse_loaded(&self, _response: &http::Response<Bytes>) -> Self::Output {
        Ok("Hello!".to_string())
    }
}

fn test_operation() -> Operation<TestOperationParser, AwsErrorRetryPolicy> {
    let req = operation::Request::new(
        http::Request::builder()
            .uri("https://test-service.test-region.amazonaws.com/")
            .body(SdkBody::from("request body"))
            .unwrap(),
    )
    .augment(|req, mut conf| {
        set_endpoint_resolver(
            &mut conf,
            Arc::new(aws_endpoint::partition::endpoint::Metadata {
                uri_template: "test-service.{region}.amazonaws.com",
                protocol: Protocol::Https,
                credential_scope: Default::default(),
                signature_versions: SignatureVersion::V4,
            }),
        );
        aws_auth::set_provider(
            &mut conf,
            SharedCredentialsProvider::new(Credentials::from_keys(
                "access_key",
                "secret_key",
                None,
            )),
        );
        conf.insert(Region::new("test-region"));
        conf.insert(OperationSigningConfig::default_config());
        conf.insert(SigningService::from_static("test-service-signing"));
        conf.insert(UNIX_EPOCH + Duration::from_secs(1613414417));
        conf.insert(AwsUserAgent::for_tests());
        Result::<_, Infallible>::Ok(req)
    })
    .unwrap();
    Operation::new(req, TestOperationParser).with_retry_policy(AwsErrorRetryPolicy::new())
}

#[cfg(any(feature = "native-tls", feature = "rustls"))]
#[test]
fn test_default_client() {
    let client = Client::https();
    let _ = client.call(test_operation());
}

#[tokio::test]
async fn e2e_test() {
    let expected_req = http::Request::builder()
        .header(USER_AGENT, "aws-sdk-rust/0.123.test os/windows/XPSP3 lang/rust/1.50.0")
        .header("x-amz-user-agent", "aws-sdk-rust/0.123.test api/test-service/0.123 os/windows/XPSP3 lang/rust/1.50.0")
        .header(AUTHORIZATION, "AWS4-HMAC-SHA256 Credential=access_key/20210215/test-region/test-service-signing/aws4_request, SignedHeaders=host;x-amz-date;x-amz-user-agent, Signature=da249491d7fe3da22c2e09cbf910f37aa5b079a3cedceff8403d0b18a7bfab75")
        .header("x-amz-date", "20210215T184017Z")
        .uri(Uri::from_static("https://test-service.test-region.amazonaws.com/"))
        .body(SdkBody::from("request body")).unwrap();
    let events = vec![(
        expected_req,
        http::Response::builder()
            .status(200)
            .body("response body")
            .unwrap(),
    )];
    let conn = TestConnection::new(events);
    let client = Client::new(conn.clone());
    let resp = client.call(test_operation()).await;
    let resp = resp.expect("successful operation");
    assert_eq!(resp, "Hello!");

    conn.assert_requests_match(&[]);
}

#[tokio::test]
async fn retry_test() {
    fn req() -> http::Request<SdkBody> {
        http::Request::builder()
            .body(SdkBody::from("request body"))
            .unwrap()
    }

    fn ok() -> http::Response<&'static str> {
        http::Response::builder()
            .status(200)
            .body("response body")
            .unwrap()
    }

    fn err() -> http::Response<&'static str> {
        http::Response::builder()
            .status(500)
            .body("response body")
            .unwrap()
    }
    // 1 failing response followed by 1 succesful response
    let events = vec![
        // First operation
        (req(), err()),
        (req(), err()),
        (req(), ok()),
        // Second operation
        (req(), err()),
        (req(), ok()),
        // Third operation will fail, only errors
        (req(), err()),
        (req(), err()),
        (req(), err()),
        (req(), err()),
        (req(), err()),
        (req(), err()),
        (req(), err()),
    ];
    let conn = TestConnection::new(events);
    let retry_config = RetryConfig::default().with_base(|| 1_f64);
    let client = Client::new(conn.clone()).with_retry_config(retry_config);
    tokio::time::pause();
    let initial = tokio::time::Instant::now();
    let resp = client
        .call(test_operation())
        .await
        .expect("successful operation");
    assert_time_passed(initial, Duration::from_secs(3));
    assert_eq!(resp, "Hello!");
    // 3 requests should have been made, 2 failing & one success
    assert_eq!(conn.requests().len(), 3);

    let initial = tokio::time::Instant::now();
    client
        .call(test_operation())
        .await
        .expect("successful operation");
    assert_time_passed(initial, Duration::from_secs(1));
    assert_eq!(conn.requests().len(), 5);
    let initial = tokio::time::Instant::now();
    let err = client
        .call(test_operation())
        .await
        .expect_err("all responses failed");
    // three more tries followed by failure
    assert_eq!(conn.requests().len(), 8);
    assert!(matches!(err, SdkError::ServiceError { .. }));
    assert_time_passed(initial, Duration::from_secs(3));
}

/// Validate that time has passed with a 5ms tolerance
///
/// This is to account for some non-determinism in the Tokio timer
fn assert_time_passed(initial: Instant, passed: Duration) {
    let now = tokio::time::Instant::now();
    let delta = now - initial;
    if (delta.as_millis() as i128 - passed.as_millis() as i128).abs() > 5 {
        assert_eq!(delta, passed)
    }
}

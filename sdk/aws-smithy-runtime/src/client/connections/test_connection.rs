/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Module with client connectors useful for testing.

use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::ConnectorError;
use aws_smithy_protocol_test::{assert_ok, validate_body, MediaType};
use aws_smithy_runtime_api::client::orchestrator::{
    BoxFallibleFut, Connection, HttpRequest, HttpResponse,
};
use aws_smithy_runtime_api::config_bag::ConfigBag;
use http::header::{HeaderName, CONTENT_TYPE};
use std::fmt::Debug;
use std::future::ready;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

/// Test Connection to capture a single request
#[derive(Debug, Clone)]
pub struct CaptureRequestHandler(Arc<Mutex<Inner>>);

#[derive(Debug)]
struct Inner {
    _response: Option<http::Response<SdkBody>>,
    _sender: Option<oneshot::Sender<HttpRequest>>,
}

/// Receiver for [`CaptureRequestHandler`](CaptureRequestHandler)
#[derive(Debug)]
pub struct CaptureRequestReceiver {
    receiver: oneshot::Receiver<HttpRequest>,
}

impl CaptureRequestReceiver {
    /// Expect that a request was sent. Returns the captured request.
    ///
    /// # Panics
    /// If no request was received
    #[track_caller]
    pub fn expect_request(mut self) -> HttpRequest {
        self.receiver.try_recv().expect("no request was received")
    }

    /// Expect that no request was captured. Panics if a request was received.
    ///
    /// # Panics
    /// If a request was received
    #[track_caller]
    pub fn expect_no_request(mut self) {
        self.receiver
            .try_recv()
            .expect_err("expected no request to be received!");
    }
}

/// Test connection used to capture a single request
///
/// If response is `None`, it will reply with a 200 response with an empty body
///
/// Example:
/// ```compile_fail
/// let (server, request) = capture_request(None);
/// let conf = aws_sdk_sts::Config::builder()
///     .http_connector(server)
///     .build();
/// let client = aws_sdk_sts::Client::from_conf(conf);
/// let _ = client.assume_role_with_saml().send().await;
/// // web identity should be unsigned
/// assert_eq!(
///     request.expect_request().headers().get("AUTHORIZATION"),
///     None
/// );
/// ```
pub fn capture_request(
    response: Option<http::Response<SdkBody>>,
) -> (CaptureRequestHandler, CaptureRequestReceiver) {
    let (tx, rx) = oneshot::channel();
    (
        CaptureRequestHandler(Arc::new(Mutex::new(Inner {
            _response: Some(response.unwrap_or_else(|| {
                http::Response::builder()
                    .status(200)
                    .body(SdkBody::empty())
                    .expect("unreachable")
            })),
            _sender: Some(tx),
        }))),
        CaptureRequestReceiver { receiver: rx },
    )
}

type ConnectVec = Vec<(HttpRequest, HttpResponse)>;

#[derive(Debug)]
pub struct ValidateRequest {
    pub expected: HttpRequest,
    pub actual: HttpRequest,
}

impl ValidateRequest {
    pub fn assert_matches(&self, ignore_headers: &[HeaderName]) {
        let (actual, expected) = (&self.actual, &self.expected);
        assert_eq!(actual.uri(), expected.uri());
        for (name, value) in expected.headers() {
            if !ignore_headers.contains(name) {
                let actual_header = actual
                    .headers()
                    .get(name)
                    .unwrap_or_else(|| panic!("Header {:?} missing", name));
                assert_eq!(
                    actual_header.to_str().unwrap(),
                    value.to_str().unwrap(),
                    "Header mismatch for {:?}",
                    name
                );
            }
        }
        let actual_str = std::str::from_utf8(actual.body().bytes().unwrap_or(&[]));
        let expected_str = std::str::from_utf8(expected.body().bytes().unwrap_or(&[]));
        let media_type = if actual
            .headers()
            .get(CONTENT_TYPE)
            .map(|v| v.to_str().unwrap().contains("json"))
            .unwrap_or(false)
        {
            MediaType::Json
        } else {
            MediaType::Other("unknown".to_string())
        };
        match (actual_str, expected_str) {
            (Ok(actual), Ok(expected)) => assert_ok(validate_body(actual, expected, media_type)),
            _ => assert_eq!(actual.body().bytes(), expected.body().bytes()),
        };
    }
}

/// TestConnection for use as a [`Connection`].
///
/// A basic test connection. It will:
/// - Respond to requests with a preloaded series of responses
/// - Record requests for future examination
#[derive(Debug)]
pub struct TestConnection {
    data: Arc<Mutex<ConnectVec>>,
    requests: Arc<Mutex<Vec<ValidateRequest>>>,
}

// Need a clone impl that ignores `B`
impl Clone for TestConnection {
    fn clone(&self) -> Self {
        TestConnection {
            data: self.data.clone(),
            requests: self.requests.clone(),
        }
    }
}

impl TestConnection {
    pub fn new(mut data: ConnectVec) -> Self {
        data.reverse();
        TestConnection {
            data: Arc::new(Mutex::new(data)),
            requests: Default::default(),
        }
    }

    pub fn requests(&self) -> impl Deref<Target = Vec<ValidateRequest>> + '_ {
        self.requests.lock().unwrap()
    }

    #[track_caller]
    pub fn assert_requests_match(&self, ignore_headers: &[HeaderName]) {
        for req in self.requests().iter() {
            req.assert_matches(ignore_headers)
        }
        let remaining_requests = self.data.lock().unwrap().len();
        let actual_requests = self.requests().len();
        assert_eq!(
            remaining_requests, 0,
            "Expected {} additional requests ({} were made)",
            remaining_requests, actual_requests
        );
    }
}

impl Connection for TestConnection {
    fn call(&self, request: &mut HttpRequest, _cfg: &ConfigBag) -> BoxFallibleFut<HttpResponse> {
        // TODO(orchestrator) Validate request

        let res = if let Some((expected, resp)) = self.data.lock().unwrap().pop() {
            let actual = try_clone_http_request(request).expect("test request is cloneable");
            self.requests
                .lock()
                .unwrap()
                .push(ValidateRequest { expected, actual });
            Ok(resp.map(SdkBody::from))
        } else {
            Err(ConnectorError::other("No more data".into(), None).into())
        };

        Box::pin(ready(res))
    }
}

pub fn try_clone_http_request(req: &http::Request<SdkBody>) -> Option<http::Request<SdkBody>> {
    let cloned_body = req.body().try_clone()?;
    let mut cloned_request = http::Request::builder()
        .uri(req.uri().clone())
        .method(req.method());
    *cloned_request
        .headers_mut()
        .expect("builder has not been modified, headers must be valid") = req.headers().clone();
    let req = cloned_request
        .body(cloned_body)
        .expect("a clone of a valid request should be a valid request");

    Some(req)
}

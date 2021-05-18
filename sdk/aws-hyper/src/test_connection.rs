/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use http::header::{HeaderName, CONTENT_TYPE};
use http::Request;
use protocol_test_helpers::{assert_ok, validate_body, MediaType};
use smithy_http::body::SdkBody;
use std::future::Ready;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tower::BoxError;

type ConnectVec<B> = Vec<(http::Request<SdkBody>, http::Response<B>)>;

pub struct ValidateRequest {
    pub expected: http::Request<SdkBody>,
    pub actual: http::Request<SdkBody>,
}

impl ValidateRequest {
    pub fn assert_matches(&self, ignore_headers: Vec<HeaderName>) {
        let (actual, expected) = (&self.actual, &self.expected);
        for (name, value) in expected.headers() {
            if !ignore_headers.contains(name) {
                let actual_header = actual
                    .headers()
                    .get(name)
                    .unwrap_or_else(|| panic!("Header {:?} missing", name));
                assert_eq!(actual_header, value, "Header mismatch for {:?}", name);
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
        assert_eq!(actual.uri(), expected.uri());
    }
}

/// TestConnection for use with a [`aws_hyper::Client`](crate::Client)
///
/// A basic test connection. It will:
/// - Response to requests with a preloaded series of responses
/// - Record requests for future examination
///
/// The generic parameter `B` is the type of the response body.
/// For more complex use cases, see [Tower Test](https://docs.rs/tower-test/0.4.0/tower_test/)
/// Usage example:
/// ```rust
/// use aws_hyper::test_connection::TestConnection;
/// use smithy_http::body::SdkBody;
/// let events = vec![(
///    http::Request::new(SdkBody::from("request body")),
///    http::Response::builder()
///        .status(200)
///        .body("response body")
///        .unwrap(),
/// )];
/// let conn = TestConnection::new(events);
/// let client = aws_hyper::Client::new(conn);
/// ```
pub struct TestConnection<B> {
    data: Arc<Mutex<ConnectVec<B>>>,
    requests: Arc<Mutex<Vec<ValidateRequest>>>,
}

// Need a clone impl that ignores `B`
impl<B> Clone for TestConnection<B> {
    fn clone(&self) -> Self {
        TestConnection {
            data: self.data.clone(),
            requests: self.requests.clone(),
        }
    }
}

impl<B> TestConnection<B> {
    pub fn new(mut data: ConnectVec<B>) -> Self {
        data.reverse();
        TestConnection {
            data: Arc::new(Mutex::new(data)),
            requests: Default::default(),
        }
    }

    pub fn requests(&self) -> impl Deref<Target = Vec<ValidateRequest>> + '_ {
        self.requests.lock().unwrap()
    }
}

impl<B: Into<hyper::Body>> tower::Service<http::Request<SdkBody>> for TestConnection<B> {
    type Response = http::Response<SdkBody>;
    type Error = BoxError;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, actual: Request<SdkBody>) -> Self::Future {
        // todo: validate request
        if let Some((expected, resp)) = self.data.lock().unwrap().pop() {
            self.requests
                .lock()
                .unwrap()
                .push(ValidateRequest { expected, actual });
            std::future::ready(Ok(resp.map(|body| SdkBody::from(body.into()))))
        } else {
            std::future::ready(Err("No more data".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_connection::TestConnection;
    use crate::{conn, Client};

    fn is_send_sync<T: Send + Sync>(_: T) {}

    #[test]
    fn construct_test_client() {
        let test_conn = TestConnection::<String>::new(vec![]);
        let client = Client::new(conn::Standard::new(test_conn));
        is_send_sync(client);
    }
}

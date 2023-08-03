/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
//! Module with client connectors useful for testing.

// TODO(docs)
#![allow(missing_docs)]

use std::fmt::{Debug, Formatter};
use std::future::Ready;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use http::header::{HeaderName, CONTENT_TYPE};
use http::Request;
use tokio::sync::oneshot;

use crate::erase::DynConnector;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::ConnectorError;
use aws_smithy_protocol_test::{assert_ok, validate_body, MediaType};

#[doc(inline)]
pub use crate::never;

/// Test Connection to capture a single request
#[derive(Debug, Clone)]
pub struct CaptureRequestHandler(Arc<Mutex<Inner>>);

#[derive(Debug)]
struct Inner {
    response: Option<http::Response<SdkBody>>,
    sender: Option<oneshot::Sender<http::Request<SdkBody>>>,
}

/// Receiver for [`CaptureRequestHandler`](CaptureRequestHandler)
#[derive(Debug)]
pub struct CaptureRequestReceiver {
    receiver: oneshot::Receiver<http::Request<SdkBody>>,
}

impl CaptureRequestReceiver {
    /// Expect that a request was sent. Returns the captured request.
    ///
    /// # Panics
    /// If no request was received
    #[track_caller]
    pub fn expect_request(mut self) -> http::Request<SdkBody> {
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

impl tower::Service<http::Request<SdkBody>> for CaptureRequestHandler {
    type Response = http::Response<SdkBody>;
    type Error = ConnectorError;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<SdkBody>) -> Self::Future {
        let mut inner = self.0.lock().unwrap();
        inner
            .sender
            .take()
            .expect("already sent")
            .send(req)
            .expect("channel not ready");
        std::future::ready(Ok(inner
            .response
            .take()
            .expect("could not handle second request")))
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
            response: Some(response.unwrap_or_else(|| {
                http::Response::builder()
                    .status(200)
                    .body(SdkBody::empty())
                    .expect("unreachable")
            })),
            sender: Some(tx),
        }))),
        CaptureRequestReceiver { receiver: rx },
    )
}

type ConnectVec<B> = Vec<(http::Request<SdkBody>, http::Response<B>)>;

#[derive(Debug)]
pub struct ValidateRequest {
    pub expected: http::Request<SdkBody>,
    pub actual: http::Request<SdkBody>,
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

/// TestConnection for use with a [`Client`](crate::Client).
///
/// A basic test connection. It will:
/// - Respond to requests with a preloaded series of responses
/// - Record requests for future examination
///
/// The generic parameter `B` is the type of the response body.
/// For more complex use cases, see [Tower Test](https://docs.rs/tower-test/0.4.0/tower_test/)
/// Usage example:
/// ```no_run
/// use aws_smithy_client::test_connection::TestConnection;
/// use aws_smithy_http::body::SdkBody;
/// let events = vec![(
///    http::Request::new(SdkBody::from("request body")),
///    http::Response::builder()
///        .status(200)
///        .body("response body")
///        .unwrap(),
/// )];
/// let conn = TestConnection::new(events);
/// let client = aws_smithy_client::Client::from(conn);
/// ```
#[derive(Debug)]
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

impl<B> tower::Service<http::Request<SdkBody>> for TestConnection<B>
where
    SdkBody: From<B>,
{
    type Response = http::Response<SdkBody>;
    type Error = ConnectorError;
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
            std::future::ready(Ok(resp.map(SdkBody::from)))
        } else {
            std::future::ready(Err(ConnectorError::other("No more data".into(), None)))
        }
    }
}

impl<B> From<TestConnection<B>> for crate::Client<TestConnection<B>, tower::layer::util::Identity>
where
    B: Send + 'static,
    SdkBody: From<B>,
{
    fn from(tc: TestConnection<B>) -> Self {
        crate::Builder::new()
            .middleware(tower::layer::util::Identity::new())
            .connector(tc)
            .build()
    }
}

/// Create a DynConnector from `Fn(http:Request) -> http::Response`
///
/// # Examples
///
/// ```rust
/// use aws_smithy_client::test_connection::infallible_connection_fn;
/// let connector = infallible_connection_fn(|_req|http::Response::builder().status(200).body("OK!").unwrap());
/// ```
pub fn infallible_connection_fn<B>(
    f: impl Fn(http::Request<SdkBody>) -> http::Response<B> + Send + Sync + 'static,
) -> DynConnector
where
    B: Into<SdkBody>,
{
    ConnectionFn::infallible(f)
}

#[derive(Clone)]
struct ConnectionFn {
    #[allow(clippy::type_complexity)]
    response: Arc<
        dyn Fn(http::Request<SdkBody>) -> Result<http::Response<SdkBody>, ConnectorError>
            + Send
            + Sync,
    >,
}

impl Debug for ConnectionFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionFn").finish()
    }
}

impl ConnectionFn {
    fn infallible<B: Into<SdkBody>>(
        f: impl Fn(http::Request<SdkBody>) -> http::Response<B> + Send + Sync + 'static,
    ) -> DynConnector {
        DynConnector::new(Self {
            response: Arc::new(move |request| Ok(f(request).map(|b| b.into()))),
        })
    }
}

impl tower::Service<http::Request<SdkBody>> for ConnectionFn {
    type Response = http::Response<SdkBody>;
    type Error = ConnectorError;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<SdkBody>) -> Self::Future {
        std::future::ready((self.response)(req))
    }
}

/// [`wire_mock`] contains utilities for mocking at the socket level
///
/// Other tools in this module actually operate at the `http::Request` / `http::Response` level. This
/// is useful, but it shortcuts the HTTP implementation (e.g. Hyper). [`wire_mock::WireLevelTestConnection`] binds
/// to an actual socket on the host
///
/// # Examples
/// ```
/// use tower::layer::util::Identity;
/// use aws_smithy_client::http_connector::ConnectorSettings;
/// use aws_smithy_client::{match_events, ev};
/// use aws_smithy_client::test_connection::wire_mock::check_matches;
/// # async fn example() {
/// use aws_smithy_client::test_connection::wire_mock::{ReplayedEvent, WireLevelTestConnection};
/// // This connection binds to a local address
/// let mock = WireLevelTestConnection::spinup(vec![
///     ReplayedEvent::status(503),
///     ReplayedEvent::status(200)
/// ]).await;
/// let client = aws_smithy_client::Client::builder()
///     .connector(mock.http_connector().connector(&ConnectorSettings::default(), None).unwrap())
///     .middleware(Identity::new())
///     .build();
/// /* do something with <client> */
/// // assert that you got the events you expected
/// match_events!(ev!(dns), ev!(connect), ev!(http(200)))(&mock.events());
/// # }
/// ```
#[cfg(feature = "wiremock")]
pub mod wire_mock {
    use bytes::Bytes;
    use http::{Request, Response};
    use hyper::client::connect::dns::Name;
    use hyper::server::conn::AddrStream;
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Server};
    use std::collections::HashSet;
    use std::convert::Infallible;
    use std::error::Error;

    use hyper::client::HttpConnector as HyperHttpConnector;
    use std::iter;
    use std::iter::Once;
    use std::net::{SocketAddr, TcpListener};
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll};

    use tokio::spawn;
    use tower::Service;

    /// An event recorded by [`WireLevelTestConnection`]
    #[derive(Debug, Clone)]
    pub enum RecordedEvent {
        DnsLookup(String),
        NewConnection,
        Response(ReplayedEvent),
    }

    type Matcher = (
        Box<dyn Fn(&RecordedEvent) -> Result<(), Box<dyn Error>>>,
        &'static str,
    );

    /// This method should only be used by the macro
    #[doc(hidden)]
    pub fn check_matches(events: &[RecordedEvent], matchers: &[Matcher]) {
        let mut events_iter = events.iter();
        let mut matcher_iter = matchers.iter();
        let mut idx = -1;
        loop {
            idx += 1;
            let bail = |err: Box<dyn Error>| panic!("failed on event {}:\n  {}", idx, err);
            match (events_iter.next(), matcher_iter.next()) {
                (Some(event), Some((matcher, _msg))) => matcher(event).unwrap_or_else(bail),
                (None, None) => return,
                (Some(event), None) => {
                    bail(format!("got {:?} but no more events were expected", event).into())
                }
                (None, Some((_expect, msg))) => {
                    bail(format!("expected {:?} but no more events were expected", msg).into())
                }
            }
        }
    }

    #[macro_export]
    macro_rules! matcher {
        ($expect:tt) => {
            (
                Box::new(
                    |event: &::aws_smithy_client::test_connection::wire_mock::RecordedEvent| {
                        if !matches!(event, $expect) {
                            return Err(format!(
                                "expected `{}` but got {:?}",
                                stringify!($expect),
                                event
                            )
                            .into());
                        }
                        Ok(())
                    },
                ),
                stringify!($expect),
            )
        };
    }

    /// Helper macro to generate a series of test expectations
    #[macro_export]
    macro_rules! match_events {
        ($( $expect:pat),*) => {
                |events| {
                    check_matches(events, &[$( ::aws_smithy_client::matcher!($expect) ),*]);
                }
        };
    }

    /// Helper to generate match expressions for events
    #[macro_export]
    macro_rules! ev {
        (http($status:expr)) => {
            ::aws_smithy_client::test_connection::wire_mock::RecordedEvent::Response(
                ReplayedEvent::HttpResponse {
                    status: $status,
                    ..
                },
            )
        };
        (dns) => {
            ::aws_smithy_client::test_connection::wire_mock::RecordedEvent::DnsLookup(_)
        };
        (connect) => {
            ::aws_smithy_client::test_connection::wire_mock::RecordedEvent::NewConnection
        };
        (timeout) => {
            ::aws_smithy_client::test_connection::wire_mock::RecordedEvent::Response(
                ReplayedEvent::Timeout,
            )
        };
    }

    pub use {ev, match_events, matcher};

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum ReplayedEvent {
        Timeout,
        HttpResponse { status: u16, body: Bytes },
    }

    impl ReplayedEvent {
        pub fn ok() -> Self {
            Self::HttpResponse {
                status: 200,
                body: Bytes::new(),
            }
        }

        pub fn with_body(body: &str) -> Self {
            Self::HttpResponse {
                status: 200,
                body: Bytes::copy_from_slice(body.as_ref()),
            }
        }

        pub fn status(status: u16) -> Self {
            Self::HttpResponse {
                status,
                body: Bytes::new(),
            }
        }
    }

    use crate::erase::boxclone::BoxFuture;
    use crate::http_connector::HttpConnector;
    use crate::hyper_ext;
    use aws_smithy_async::future::never::Never;
    use tokio::sync::oneshot;

    /// Test connection that starts a server bound to 0.0.0.0
    ///
    /// See the [module docs](crate::test_connection::wire_mock) for a usage example.
    ///
    /// Usage:
    /// - Call [`WireLevelTestConnection::spinup`] to start the server
    /// - Use [`WireLevelTestConnection::http_connector`] or [`dns_resolver`](WireLevelTestConnection::dns_resolver) to configure your client.
    /// - Make requests to [`endpoint_url`](WireLevelTestConnection::endpoint_url).
    /// - Once the test is complete, retrieve a list of events from [`WireLevelTestConnection::events`]
    #[derive(Debug)]
    pub struct WireLevelTestConnection {
        event_log: Arc<Mutex<Vec<RecordedEvent>>>,
        bind_addr: SocketAddr,
        // when the sender is dropped, that stops the server
        shutdown_hook: oneshot::Sender<()>,
    }

    impl WireLevelTestConnection {
        pub async fn spinup(mut response_events: Vec<ReplayedEvent>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let (tx, rx) = oneshot::channel();
            let listener_addr = listener.local_addr().unwrap();
            response_events.reverse();
            let response_events = Arc::new(Mutex::new(response_events));
            let handler_events = response_events;
            let wire_events = Arc::new(Mutex::new(vec![]));
            let wire_log_for_service = wire_events.clone();
            let poisoned_conns: Arc<Mutex<HashSet<SocketAddr>>> = Default::default();
            let make_service = make_service_fn(move |connection: &AddrStream| {
                let poisoned_conns = poisoned_conns.clone();
                let events = handler_events.clone();
                let wire_log = wire_log_for_service.clone();
                let remote_addr = connection.remote_addr();
                tracing::info!("established connection: {:?}", connection);
                wire_log.lock().unwrap().push(RecordedEvent::NewConnection);
                async move {
                    Ok::<_, Infallible>(service_fn(move |_: Request<hyper::Body>| {
                        if poisoned_conns.lock().unwrap().contains(&remote_addr) {
                            tracing::error!("poisoned connection {:?} was reused!", &remote_addr);
                            panic!("poisoned connection was reused!");
                        }
                        let next_event = events.clone().lock().unwrap().pop();
                        let wire_log = wire_log.clone();
                        let poisoned_conns = poisoned_conns.clone();
                        async move {
                            let next_event = next_event
                                .unwrap_or_else(|| panic!("no more events! Log: {:?}", wire_log));
                            wire_log
                                .lock()
                                .unwrap()
                                .push(RecordedEvent::Response(next_event.clone()));
                            if next_event == ReplayedEvent::Timeout {
                                tracing::info!("{} is poisoned", remote_addr);
                                poisoned_conns.lock().unwrap().insert(remote_addr);
                            }
                            tracing::debug!("replying with {:?}", next_event);
                            let event = generate_response_event(next_event).await;
                            dbg!(event)
                        }
                    }))
                }
            });
            let server = Server::from_tcp(listener)
                .unwrap()
                .serve(make_service)
                .with_graceful_shutdown(async {
                    rx.await.ok();
                    tracing::info!("server shutdown!");
                });
            spawn(async move { server.await });
            Self {
                event_log: wire_events,
                bind_addr: listener_addr,
                shutdown_hook: tx,
            }
        }

        /// Retrieve the events recorded by this connection
        pub fn events(&self) -> Vec<RecordedEvent> {
            self.event_log.lock().unwrap().clone()
        }

        fn bind_addr(&self) -> SocketAddr {
            self.bind_addr
        }

        pub fn dns_resolver(&self) -> LoggingDnsResolver {
            let event_log = self.event_log.clone();
            let bind_addr = self.bind_addr;
            LoggingDnsResolver {
                log: event_log,
                socket_addr: bind_addr,
            }
        }

        /// Prebuilt HTTP connector with correctly wired DNS resolver
        ///
        /// **Note**: This must be used in tandem with [`Self::dns_resolver`]
        pub fn http_connector(&self) -> HttpConnector {
            let http_connector = HyperHttpConnector::new_with_resolver(self.dns_resolver());
            hyper_ext::Adapter::builder().build(http_connector).into()
        }

        /// Endpoint to use when connecting
        ///
        /// This works in tandem with the [`Self::dns_resolver`] to bind to the correct local IP Address
        pub fn endpoint_url(&self) -> String {
            format!(
                "http://this-url-is-converted-to-localhost.com:{}",
                self.bind_addr().port()
            )
        }

        pub fn shutdown(self) {
            let _ = self.shutdown_hook.send(());
        }
    }

    async fn generate_response_event(event: ReplayedEvent) -> Result<Response<Body>, Infallible> {
        let resp = match event {
            ReplayedEvent::HttpResponse { status, body } => http::Response::builder()
                .status(status)
                .body(hyper::Body::from(body))
                .unwrap(),
            ReplayedEvent::Timeout => {
                Never::new().await;
                unreachable!()
            }
        };
        Ok::<_, Infallible>(resp)
    }

    /// DNS resolver that keeps a log of all lookups
    ///
    /// Regardless of what hostname is requested, it will always return the same socket address.
    #[derive(Clone, Debug)]
    pub struct LoggingDnsResolver {
        log: Arc<Mutex<Vec<RecordedEvent>>>,
        socket_addr: SocketAddr,
    }

    impl Service<Name> for LoggingDnsResolver {
        type Response = Once<SocketAddr>;
        type Error = Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: Name) -> Self::Future {
            let sock_addr = self.socket_addr;
            let log = self.log.clone();
            Box::pin(async move {
                println!("looking up {:?}, replying with {:?}", req, sock_addr);
                log.lock()
                    .unwrap()
                    .push(RecordedEvent::DnsLookup(req.to_string()));
                Ok(iter::once(sock_addr))
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use tower::Service;

    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::result::ConnectorError;

    use crate::bounds::SmithyConnector;
    use crate::test_connection::{capture_request, never::NeverService, TestConnection};
    use crate::Client;

    fn is_send_sync<T: Send + Sync>(_: T) {}

    #[test]
    fn construct_test_client() {
        let test_conn = TestConnection::<String>::new(vec![]);
        let client: Client<_, _, _> = test_conn.into();
        is_send_sync(client);
    }

    fn is_a_connector<T>(_: &T)
    where
        T: SmithyConnector,
    {
    }

    fn quacks_like_a_connector<T>(_: &T)
    where
        T: Service<http::Request<SdkBody>, Response = http::Response<SdkBody>>
            + Send
            + Sync
            + Clone
            + 'static,
        T::Error: Into<ConnectorError> + Send + Sync + 'static,
        T::Future: Send + 'static,
    {
    }

    #[test]
    fn oneshot_client() {
        let (tx, _rx) = capture_request(None);
        quacks_like_a_connector(&tx);
        is_a_connector(&tx)
    }

    #[test]
    fn never_test() {
        is_a_connector(&NeverService::<
            http::Request<SdkBody>,
            http::Response<SdkBody>,
            ConnectorError,
        >::new())
    }
}

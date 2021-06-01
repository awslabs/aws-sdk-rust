/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

pub mod conn;
mod retry;
#[cfg(feature = "test-util")]
pub mod test_connection;

pub use retry::RetryConfig;

use crate::conn::Standard;
use crate::retry::RetryHandlerFactory;
use aws_endpoint::AwsEndpointStage;
use aws_http::user_agent::UserAgentStage;
use aws_sig_auth::middleware::SigV4SigningStage;
use aws_sig_auth::signer::SigV4Signer;
use smithy_http::body::SdkBody;
use smithy_http::operation::Operation;
use smithy_http::response::ParseHttpResponse;
pub use smithy_http::result::{SdkError, SdkSuccess};
use smithy_http::retry::ClassifyResponse;
use smithy_http_tower::dispatch::DispatchLayer;
use smithy_http_tower::map_request::MapRequestLayer;
use smithy_http_tower::parse_response::ParseResponseLayer;
use smithy_types::retry::ProvideErrorKind;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Formatter};
use tower::{Service, ServiceBuilder, ServiceExt};

type BoxError = Box<dyn Error + Send + Sync>;
pub type StandardClient = Client<conn::Standard>;

/// AWS Service Client
///
/// Hyper-based AWS Service Client. Most customers will want to construct a client with
/// [`Client::https()`](Client::https). For testing & other more advanced use cases, a custom
/// connector may be used via [`Client::new(connector)`](Client::new).
///
/// The internal connector must implement the following trait bound to be used to dispatch requests:
/// ```rust,ignore
///    S: Service<http::Request<SdkBody>, Response = http::Response<hyper::Body>>
///        + Send
///        + Clone
///        + 'static,
///    S::Error: Into<BoxError> + Send + Sync + 'static,
///    S::Future: Send + 'static,
/// ```
pub struct Client<S> {
    inner: S,
    retry_handler: RetryHandlerFactory,
}

impl<S> Debug for Client<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut formatter = f.debug_struct("Client");
        formatter.field("retry_handler", &self.retry_handler);
        formatter.finish()
    }
}

impl<S> Client<S> {
    /// Construct a new `Client` with a custom connector
    pub fn new(connector: S) -> Self {
        Client {
            inner: connector,
            retry_handler: RetryHandlerFactory::new(RetryConfig::default()),
        }
    }

    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_handler.with_config(retry_config);
        self
    }
}

impl Client<Standard> {
    /// Construct an `https` based client
    #[cfg(any(feature = "native-tls", feature = "rustls"))]
    pub fn https() -> StandardClient {
        Client {
            inner: Standard::https(),
            retry_handler: RetryHandlerFactory::new(RetryConfig::default()),
        }
    }
}

impl<S> Client<S>
where
    S: Service<http::Request<SdkBody>, Response = http::Response<SdkBody>> + Send + Clone + 'static,
    S::Error: Into<BoxError> + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    /// Dispatch this request to the network
    ///
    /// For ergonomics, this does not include the raw response for successful responses. To
    /// access the raw response use `call_raw`.
    pub async fn call<O, T, E, Retry>(&self, input: Operation<O, Retry>) -> Result<T, SdkError<E>>
    where
        O: ParseHttpResponse<SdkBody, Output = Result<T, E>> + Send + Sync + Clone + 'static,
        E: Error + ProvideErrorKind,
        Retry: ClassifyResponse<SdkSuccess<T>, SdkError<E>>,
    {
        self.call_raw(input).await.map(|res| res.parsed)
    }

    /// Dispatch this request to the network
    ///
    /// The returned result contains the raw HTTP response which can be useful for debugging or implementing
    /// unsupported features.
    pub async fn call_raw<O, R, E, Retry>(
        &self,
        input: Operation<O, Retry>,
    ) -> Result<SdkSuccess<R>, SdkError<E>>
    where
        O: ParseHttpResponse<SdkBody, Output = Result<R, E>> + Send + Sync + Clone + 'static,
        E: Error + ProvideErrorKind,
        Retry: ClassifyResponse<SdkSuccess<R>, SdkError<E>>,
    {
        let signer = MapRequestLayer::for_mapper(SigV4SigningStage::new(SigV4Signer::new()));
        let endpoint_resolver = MapRequestLayer::for_mapper(AwsEndpointStage);
        let user_agent = MapRequestLayer::for_mapper(UserAgentStage::new());
        let inner = self.inner.clone();
        let mut svc = ServiceBuilder::new()
            // Create a new request-scoped policy
            .retry(self.retry_handler.new_handler())
            .layer(ParseResponseLayer::<O, Retry>::new())
            // These layers can be considered as occuring in order, that is:
            // 1. Resolve an endpoint
            // 2. Add a user agent
            // 3. Sign
            // 4. Dispatch over the wire
            .layer(endpoint_resolver)
            .layer(user_agent)
            .layer(signer)
            .layer(DispatchLayer::new())
            .service(inner);
        svc.ready().await?.call(input).await
    }
}

#[cfg(test)]
mod tests {

    #[cfg(any(feature = "rustls", feature = "native-tls"))]
    #[test]
    fn construct_default_client() {
        let c = crate::Client::https();
        fn is_send_sync<T: Send + Sync>(_c: T) {}
        is_send_sync(c);
    }

    #[cfg(any(feature = "rustls", feature = "native-tls"))]
    #[test]
    fn client_debug_includes_retry_info() {
        let client = crate::Client::https();
        let s = format!("{:?}", client);
        assert!(s.contains("RetryConfig"));
        assert!(s.contains("quota_available"));
    }
}

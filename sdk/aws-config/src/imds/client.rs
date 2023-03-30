/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Raw IMDSv2 Client
//!
//! Client for direct access to IMDSv2.

use crate::connector::expect_connector;
use crate::imds::client::error::{BuildError, ImdsError, InnerImdsError, InvalidEndpointMode};
use crate::imds::client::token::TokenMiddleware;
use crate::provider_config::ProviderConfig;
use crate::PKG_VERSION;
use aws_http::user_agent::{ApiMetadata, AwsUserAgent, UserAgentStage};
use aws_sdk_sso::config::timeout::TimeoutConfig;
use aws_smithy_client::http_connector::ConnectorSettings;
use aws_smithy_client::{erase::DynConnector, SdkSuccess};
use aws_smithy_client::{retry, SdkError};
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::endpoint::apply_endpoint;
use aws_smithy_http::operation;
use aws_smithy_http::operation::{Metadata, Operation};
use aws_smithy_http::response::ParseStrictResponse;
use aws_smithy_http::retry::ClassifyRetry;
use aws_smithy_http_tower::map_request::{
    AsyncMapRequestLayer, AsyncMapRequestService, MapRequestLayer, MapRequestService,
};
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_smithy_types::retry::{ErrorKind, RetryKind};
use aws_types::os_shim_internal::Env;
use bytes::Bytes;
use http::{Response, Uri};
use std::borrow::Cow;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OnceCell;

pub mod error;
mod token;

// 6 hours
const DEFAULT_TOKEN_TTL: Duration = Duration::from_secs(21_600);
const DEFAULT_ATTEMPTS: u32 = 4;
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(1);
const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(1);

fn user_agent() -> AwsUserAgent {
    AwsUserAgent::new_from_environment(Env::real(), ApiMetadata::new("imds", PKG_VERSION))
}

/// IMDSv2 Client
///
/// Client for IMDSv2. This client handles fetching tokens, retrying on failure, and token
/// caching according to the specified token TTL.
///
/// _Note: This client ONLY supports IMDSv2. It will not fallback to IMDSv1. See
/// [transitioning to IMDSv2](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-instance-metadata-service.html#instance-metadata-transition-to-version-2)
/// for more information._
///
/// **Note**: When running in a Docker container, all network requests will incur an additional hop. When combined with the default IMDS hop limit of 1, this will cause requests to IMDS to timeout! To fix this issue, you'll need to set the following instance metadata settings :
/// ```txt
/// amazonec2-metadata-token=required
/// amazonec2-metadata-token-response-hop-limit=2
/// ```
///
/// On an instance that is already running, these can be set with [ModifyInstanceMetadataOptions](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_ModifyInstanceMetadataOptions.html). On a new instance, these can be set with the `MetadataOptions` field on [RunInstances](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_RunInstances.html).
///
/// For more information about IMDSv2 vs. IMDSv1 see [this guide](https://docs.aws.amazon.com/AWSEC2/latest/WindowsGuide/configuring-instance-metadata-service.html)
///
/// # Client Configuration
/// The IMDS client can load configuration explicitly, via environment variables, or via
/// `~/.aws/config`. It will first attempt to resolve an endpoint override. If no endpoint
/// override exists, it will attempt to resolve an [`EndpointMode`]. If no
/// [`EndpointMode`] override exists, it will fallback to [`IpV4`](EndpointMode::IpV4). An exhaustive
/// list is below:
///
/// ## Endpoint configuration list
/// 1. Explicit configuration of `Endpoint` via the [builder](Builder):
/// ```no_run
/// use aws_config::imds::client::Client;
/// use http::Uri;
/// # async fn docs() {
/// let client = Client::builder()
///   .endpoint(Uri::from_static("http://customidms:456/"))
///   .build()
///   .await;
/// # }
/// ```
///
/// 2. The `AWS_EC2_METADATA_SERVICE_ENDPOINT` environment variable. Note: If this environment variable
/// is set, it MUST contain to a valid URI or client construction will fail.
///
/// 3. The `ec2_metadata_service_endpoint` field in `~/.aws/config`:
/// ```ini
/// [default]
/// # ... other configuration
/// ec2_metadata_service_endpoint = http://my-custom-endpoint:444
/// ```
///
/// 4. An explicitly set endpoint mode:
/// ```no_run
/// use aws_config::imds::client::{Client, EndpointMode};
/// # async fn docs() {
/// let client = Client::builder().endpoint_mode(EndpointMode::IpV6).build().await;
/// # }
/// ```
///
/// 5. An [endpoint mode](EndpointMode) loaded from the `AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE` environment
/// variable. Valid values: `IPv4`, `IPv6`
///
/// 6. An [endpoint mode](EndpointMode) loaded from the `ec2_metadata_service_endpoint_mode` field in
/// `~/.aws/config`:
/// ```ini
/// [default]
/// # ... other configuration
/// ec2_metadata_service_endpoint_mode = IPv4
/// ```
///
/// 7. The default value of `http://169.254.169.254` will be used.
///
#[derive(Clone, Debug)]
pub struct Client {
    inner: Arc<ClientInner>,
}

#[derive(Debug)]
struct ClientInner {
    endpoint: Uri,
    smithy_client: aws_smithy_client::Client<DynConnector, ImdsMiddleware>,
}

/// Client where build is sync, but usage is async
///
/// Building an imds::Client is actually an async operation, however, for credentials and region
/// providers, we want build to always be a synchronous operation. This allows building to be deferred
/// and cached until request time.
#[derive(Debug)]
pub(super) struct LazyClient {
    client: OnceCell<Result<Client, BuildError>>,
    builder: Builder,
}

impl LazyClient {
    pub(super) fn from_ready_client(client: Client) -> Self {
        Self {
            client: OnceCell::from(Ok(client)),
            // the builder will never be used in this case
            builder: Builder::default(),
        }
    }
    pub(super) async fn client(&self) -> Result<&Client, &BuildError> {
        let builder = &self.builder;
        self.client
            // the clone will only happen once when we actually construct it for the first time,
            // after that, we will use the cache.
            .get_or_init(|| async {
                let client = builder.clone().build().await;
                if let Err(err) = &client {
                    tracing::warn!(err = %DisplayErrorContext(err), "failed to create IMDS client")
                }
                client
            })
            .await
            .as_ref()
    }
}

impl Client {
    /// IMDS client builder
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Retrieve information from IMDS
    ///
    /// This method will handle loading and caching a session token, combining the `path` with the
    /// configured IMDS endpoint, and retrying potential errors.
    ///
    /// For more information about IMDSv2 methods and functionality, see
    /// [Instance metadata and user data](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-instance-metadata.html)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use aws_config::imds::client::Client;
    /// # async fn docs() {
    /// let client = Client::builder().build().await.expect("valid client");
    /// let ami_id = client
    ///   .get("/latest/meta-data/ami-id")
    ///   .await
    ///   .expect("failure communicating with IMDS");
    /// # }
    /// ```
    pub async fn get(&self, path: &str) -> Result<String, ImdsError> {
        let operation = self.make_operation(path)?;
        self.inner
            .smithy_client
            .call(operation)
            .await
            .map_err(|err| match err {
                SdkError::ConstructionFailure(_) if err.source().is_some() => {
                    match err.into_source().map(|e| e.downcast::<ImdsError>()) {
                        Ok(Ok(token_failure)) => *token_failure,
                        Ok(Err(err)) => ImdsError::unexpected(err),
                        Err(err) => ImdsError::unexpected(err),
                    }
                }
                SdkError::ConstructionFailure(_) => ImdsError::unexpected(err),
                SdkError::ServiceError(context) => match context.err() {
                    InnerImdsError::InvalidUtf8 => {
                        ImdsError::unexpected("IMDS returned invalid UTF-8")
                    }
                    InnerImdsError::BadStatus => {
                        ImdsError::error_response(context.into_raw().into_parts().0)
                    }
                },
                SdkError::TimeoutError(_)
                | SdkError::DispatchFailure(_)
                | SdkError::ResponseError(_) => ImdsError::io_error(err),
                _ => ImdsError::unexpected(err),
            })
    }

    /// Creates a aws_smithy_http Operation to for `path`
    /// - Convert the path to a URI
    /// - Set the base endpoint on the URI
    /// - Add a user agent
    fn make_operation(
        &self,
        path: &str,
    ) -> Result<Operation<ImdsGetResponseHandler, ImdsResponseRetryClassifier>, ImdsError> {
        let mut base_uri: Uri = path.parse().map_err(|_| {
            ImdsError::unexpected("IMDS path was not a valid URI. Hint: does it begin with `/`?")
        })?;
        apply_endpoint(&mut base_uri, &self.inner.endpoint, None).map_err(ImdsError::unexpected)?;
        let request = http::Request::builder()
            .uri(base_uri)
            .body(SdkBody::empty())
            .expect("valid request");
        let mut request = operation::Request::new(request);
        request.properties_mut().insert(user_agent());
        Ok(Operation::new(request, ImdsGetResponseHandler)
            .with_metadata(Metadata::new("get", "imds"))
            .with_retry_classifier(ImdsResponseRetryClassifier))
    }
}

/// IMDS Middleware
///
/// The IMDS middleware includes a token-loader & a UserAgent stage
#[derive(Clone, Debug)]
struct ImdsMiddleware {
    token_loader: TokenMiddleware,
}

impl<S> tower::Layer<S> for ImdsMiddleware {
    type Service = AsyncMapRequestService<MapRequestService<S, UserAgentStage>, TokenMiddleware>;

    fn layer(&self, inner: S) -> Self::Service {
        AsyncMapRequestLayer::for_mapper(self.token_loader.clone())
            .layer(MapRequestLayer::for_mapper(UserAgentStage::new()).layer(inner))
    }
}

#[derive(Copy, Clone)]
struct ImdsGetResponseHandler;

impl ParseStrictResponse for ImdsGetResponseHandler {
    type Output = Result<String, InnerImdsError>;

    fn parse(&self, response: &Response<Bytes>) -> Self::Output {
        if response.status().is_success() {
            std::str::from_utf8(response.body().as_ref())
                .map(|data| data.to_string())
                .map_err(|_| InnerImdsError::InvalidUtf8)
        } else {
            Err(InnerImdsError::BadStatus)
        }
    }
}

/// IMDSv2 Endpoint Mode
///
/// IMDS can be accessed in two ways:
/// 1. Via the IpV4 endpoint: `http://169.254.169.254`
/// 2. Via the Ipv6 endpoint: `http://[fd00:ec2::254]`
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum EndpointMode {
    /// IpV4 mode: `http://169.254.169.254`
    ///
    /// This mode is the default unless otherwise specified.
    IpV4,
    /// IpV6 mode: `http://[fd00:ec2::254]`
    IpV6,
}

impl FromStr for EndpointMode {
    type Err = InvalidEndpointMode;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            _ if value.eq_ignore_ascii_case("ipv4") => Ok(EndpointMode::IpV4),
            _ if value.eq_ignore_ascii_case("ipv6") => Ok(EndpointMode::IpV6),
            other => Err(InvalidEndpointMode::new(other.to_owned())),
        }
    }
}

impl EndpointMode {
    /// IMDS URI for this endpoint mode
    fn endpoint(&self) -> Uri {
        match self {
            EndpointMode::IpV4 => Uri::from_static("http://169.254.169.254"),
            EndpointMode::IpV6 => Uri::from_static("http://[fd00:ec2::254]"),
        }
    }
}

/// IMDSv2 Client Builder
#[derive(Default, Debug, Clone)]
pub struct Builder {
    max_attempts: Option<u32>,
    endpoint: Option<EndpointSource>,
    mode_override: Option<EndpointMode>,
    token_ttl: Option<Duration>,
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    config: Option<ProviderConfig>,
}

impl Builder {
    /// Override the number of retries for fetching tokens & metadata
    ///
    /// By default, 4 attempts will be made.
    pub fn max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = Some(max_attempts);
        self
    }

    /// Configure generic options of the [`Client`]
    ///
    /// # Examples
    /// ```no_run
    /// # async fn test() {
    /// use aws_config::imds::Client;
    /// use aws_config::provider_config::ProviderConfig;
    ///
    /// let provider = Client::builder()
    ///     .configure(&ProviderConfig::with_default_region().await)
    ///     .build();
    /// # }
    /// ```
    pub fn configure(mut self, provider_config: &ProviderConfig) -> Self {
        self.config = Some(provider_config.clone());
        self
    }

    /// Override the endpoint for the [`Client`]
    ///
    /// By default, the client will resolve an endpoint from the environment, AWS config, and endpoint mode.
    ///
    /// See [`Client`] for more information.
    pub fn endpoint(mut self, endpoint: impl Into<Uri>) -> Self {
        self.endpoint = Some(EndpointSource::Explicit(endpoint.into()));
        self
    }

    /// Override the endpoint mode for [`Client`]
    ///
    /// * When set to [`IpV4`](EndpointMode::IpV4), the endpoint will be `http://169.254.169.254`.
    /// * When set to [`IpV6`](EndpointMode::IpV6), the endpoint will be `http://[fd00:ec2::254]`.
    pub fn endpoint_mode(mut self, mode: EndpointMode) -> Self {
        self.mode_override = Some(mode);
        self
    }

    /// Override the time-to-live for the session token
    ///
    /// Requests to IMDS utilize a session token for authentication. By default, session tokens last
    /// for 6 hours. When the TTL for the token expires, a new token must be retrieved from the
    /// metadata service.
    pub fn token_ttl(mut self, ttl: Duration) -> Self {
        self.token_ttl = Some(ttl);
        self
    }

    /// Override the connect timeout for IMDS
    ///
    /// This value defaults to 1 second
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Override the read timeout for IMDS
    ///
    /// This value defaults to 1 second
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = Some(timeout);
        self
    }

    /* TODO(https://github.com/awslabs/aws-sdk-rust/issues/339): Support customizing the port explicitly */
    /*
    pub fn port(mut self, port: u32) -> Self {
        self.port_override = Some(port);
        self
    }*/

    pub(super) fn build_lazy(self) -> LazyClient {
        LazyClient {
            client: OnceCell::new(),
            builder: self,
        }
    }

    /// Build an IMDSv2 Client
    pub async fn build(self) -> Result<Client, BuildError> {
        let config = self.config.unwrap_or_default();
        let timeout_config = TimeoutConfig::builder()
            .connect_timeout(self.connect_timeout.unwrap_or(DEFAULT_CONNECT_TIMEOUT))
            .read_timeout(self.read_timeout.unwrap_or(DEFAULT_READ_TIMEOUT))
            .build();
        let connector_settings = ConnectorSettings::from_timeout_config(&timeout_config);
        let connector = expect_connector(config.connector(&connector_settings));
        let endpoint_source = self
            .endpoint
            .unwrap_or_else(|| EndpointSource::Env(config.clone()));
        let endpoint = endpoint_source.endpoint(self.mode_override).await?;
        let retry_config = retry::Config::default()
            .with_max_attempts(self.max_attempts.unwrap_or(DEFAULT_ATTEMPTS));
        let token_loader = token::TokenMiddleware::new(
            connector.clone(),
            config.time_source(),
            endpoint.clone(),
            self.token_ttl.unwrap_or(DEFAULT_TOKEN_TTL),
            retry_config.clone(),
            timeout_config.clone(),
            config.sleep(),
        );
        let middleware = ImdsMiddleware { token_loader };
        let mut smithy_builder = aws_smithy_client::Client::builder()
            .connector(connector.clone())
            .middleware(middleware)
            .retry_config(retry_config)
            .operation_timeout_config(timeout_config.into());
        smithy_builder.set_sleep_impl(config.sleep());
        let smithy_client = smithy_builder.build();

        let client = Client {
            inner: Arc::new(ClientInner {
                endpoint,
                smithy_client,
            }),
        };
        Ok(client)
    }
}

mod env {
    pub(super) const ENDPOINT: &str = "AWS_EC2_METADATA_SERVICE_ENDPOINT";
    pub(super) const ENDPOINT_MODE: &str = "AWS_EC2_METADATA_SERVICE_ENDPOINT_MODE";
}

mod profile_keys {
    pub(super) const ENDPOINT: &str = "ec2_metadata_service_endpoint";
    pub(super) const ENDPOINT_MODE: &str = "ec2_metadata_service_endpoint_mode";
}

/// Endpoint Configuration Abstraction
#[derive(Debug, Clone)]
enum EndpointSource {
    Explicit(Uri),
    Env(ProviderConfig),
}

impl EndpointSource {
    async fn endpoint(&self, mode_override: Option<EndpointMode>) -> Result<Uri, BuildError> {
        match self {
            EndpointSource::Explicit(uri) => {
                if mode_override.is_some() {
                    tracing::warn!(endpoint = ?uri, mode = ?mode_override,
                        "Endpoint mode override was set in combination with an explicit endpoint. \
                        The mode override will be ignored.")
                }
                Ok(uri.clone())
            }
            EndpointSource::Env(conf) => {
                let env = conf.env();
                // load an endpoint override from the environment
                let profile = conf.profile().await;
                let uri_override = if let Ok(uri) = env.get(env::ENDPOINT) {
                    Some(Cow::Owned(uri))
                } else {
                    profile
                        .and_then(|profile| profile.get(profile_keys::ENDPOINT))
                        .map(Cow::Borrowed)
                };
                if let Some(uri) = uri_override {
                    return Uri::try_from(uri.as_ref()).map_err(BuildError::invalid_endpoint_uri);
                }

                // if not, load a endpoint mode from the environment
                let mode = if let Some(mode) = mode_override {
                    mode
                } else if let Ok(mode) = env.get(env::ENDPOINT_MODE) {
                    mode.parse::<EndpointMode>()
                        .map_err(BuildError::invalid_endpoint_mode)?
                } else if let Some(mode) = profile.and_then(|p| p.get(profile_keys::ENDPOINT_MODE))
                {
                    mode.parse::<EndpointMode>()
                        .map_err(BuildError::invalid_endpoint_mode)?
                } else {
                    EndpointMode::IpV4
                };

                Ok(mode.endpoint())
            }
        }
    }
}

#[derive(Clone)]
struct ImdsResponseRetryClassifier;

impl ImdsResponseRetryClassifier {
    fn classify(response: &operation::Response) -> RetryKind {
        let status = response.http().status();
        match status {
            _ if status.is_server_error() => RetryKind::Error(ErrorKind::ServerError),
            // 401 indicates that the token has expired, this is retryable
            _ if status.as_u16() == 401 => RetryKind::Error(ErrorKind::ServerError),
            // This catch-all includes successful responses that fail to parse. These should not be retried.
            _ => RetryKind::UnretryableFailure,
        }
    }
}

/// IMDS Response Retry Classifier
///
/// Possible status codes:
/// - 200 (OK)
/// - 400 (Missing or invalid parameters) **Not Retryable**
/// - 401 (Unauthorized, expired token) **Retryable**
/// - 403 (IMDS disabled): **Not Retryable**
/// - 404 (Not found): **Not Retryable**
/// - >=500 (server error): **Retryable**
impl<T, E> ClassifyRetry<SdkSuccess<T>, SdkError<E>> for ImdsResponseRetryClassifier {
    fn classify_retry(&self, response: Result<&SdkSuccess<T>, &SdkError<E>>) -> RetryKind {
        match response {
            Ok(_) => RetryKind::Unnecessary,
            Err(SdkError::ResponseError(context)) => Self::classify(context.raw()),
            Err(SdkError::ServiceError(context)) => Self::classify(context.raw()),
            _ => RetryKind::UnretryableFailure,
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::imds::client::{Client, EndpointMode, ImdsResponseRetryClassifier};
    use crate::provider_config::ProviderConfig;
    use aws_credential_types::time_source::{TestingTimeSource, TimeSource};
    use aws_smithy_async::rt::sleep::TokioSleep;
    use aws_smithy_client::erase::DynConnector;
    use aws_smithy_client::test_connection::{capture_request, TestConnection};
    use aws_smithy_client::{SdkError, SdkSuccess};
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::operation;
    use aws_smithy_types::retry::RetryKind;
    use aws_types::os_shim_internal::{Env, Fs};
    use http::header::USER_AGENT;
    use http::Uri;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::error::Error;
    use std::io;
    use std::time::{Duration, UNIX_EPOCH};
    use tracing_test::traced_test;

    macro_rules! assert_full_error_contains {
        ($err:expr, $contains:expr) => {
            let err = $err;
            let message = format!(
                "{}",
                aws_smithy_types::error::display::DisplayErrorContext(&err)
            );
            assert!(
                message.contains($contains),
                "Error message '{message}' didn't contain text '{}'",
                $contains
            );
        };
    }

    const TOKEN_A: &str = "AQAEAFTNrA4eEGx0AQgJ1arIq_Cc-t4tWt3fB0Hd8RKhXlKc5ccvhg==";
    const TOKEN_B: &str = "alternatetoken==";

    pub(crate) fn token_request(base: &str, ttl: u32) -> http::Request<SdkBody> {
        http::Request::builder()
            .uri(format!("{}/latest/api/token", base))
            .header("x-aws-ec2-metadata-token-ttl-seconds", ttl)
            .method("PUT")
            .body(SdkBody::empty())
            .unwrap()
    }

    pub(crate) fn token_response(ttl: u32, token: &'static str) -> http::Response<&'static str> {
        http::Response::builder()
            .status(200)
            .header("X-aws-ec2-metadata-token-ttl-seconds", ttl)
            .body(token)
            .unwrap()
    }

    pub(crate) fn imds_request(path: &'static str, token: &str) -> http::Request<SdkBody> {
        http::Request::builder()
            .uri(Uri::from_static(path))
            .method("GET")
            .header("x-aws-ec2-metadata-token", token)
            .body(SdkBody::empty())
            .unwrap()
    }

    pub(crate) fn imds_response(body: &'static str) -> http::Response<&'static str> {
        http::Response::builder().status(200).body(body).unwrap()
    }

    pub(crate) async fn make_client<T>(conn: &TestConnection<T>) -> super::Client
    where
        SdkBody: From<T>,
        T: Send + 'static,
    {
        tokio::time::pause();
        super::Client::builder()
            .configure(
                &ProviderConfig::no_configuration()
                    .with_sleep(TokioSleep::new())
                    .with_http_connector(DynConnector::new(conn.clone())),
            )
            .build()
            .await
            .expect("valid client")
    }

    #[tokio::test]
    async fn client_caches_token() {
        let connection = TestConnection::new(vec![
            (
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A),
            ),
            (
                imds_request("http://169.254.169.254/latest/metadata", TOKEN_A),
                imds_response(r#"test-imds-output"#),
            ),
            (
                imds_request("http://169.254.169.254/latest/metadata2", TOKEN_A),
                imds_response("output2"),
            ),
        ]);
        let client = make_client(&connection).await;
        // load once
        let metadata = client.get("/latest/metadata").await.expect("failed");
        assert_eq!(metadata, "test-imds-output");
        // load again: the cached token should be used
        let metadata = client.get("/latest/metadata2").await.expect("failed");
        assert_eq!(metadata, "output2");
        connection.assert_requests_match(&[]);
    }

    #[tokio::test]
    async fn token_can_expire() {
        let connection = TestConnection::new(vec![
            (
                token_request("http://[fd00:ec2::254]", 600),
                token_response(600, TOKEN_A),
            ),
            (
                imds_request("http://[fd00:ec2::254]/latest/metadata", TOKEN_A),
                imds_response(r#"test-imds-output1"#),
            ),
            (
                token_request("http://[fd00:ec2::254]", 600),
                token_response(600, TOKEN_B),
            ),
            (
                imds_request("http://[fd00:ec2::254]/latest/metadata", TOKEN_B),
                imds_response(r#"test-imds-output2"#),
            ),
        ]);
        let mut time_source = TestingTimeSource::new(UNIX_EPOCH);
        tokio::time::pause();
        let client = super::Client::builder()
            .configure(
                &ProviderConfig::no_configuration()
                    .with_http_connector(DynConnector::new(connection.clone()))
                    .with_time_source(TimeSource::testing(&time_source))
                    .with_sleep(TokioSleep::new()),
            )
            .endpoint_mode(EndpointMode::IpV6)
            .token_ttl(Duration::from_secs(600))
            .build()
            .await
            .expect("valid client");

        let resp1 = client.get("/latest/metadata").await.expect("success");
        // now the cached credential has expired
        time_source.advance(Duration::from_secs(600));
        let resp2 = client.get("/latest/metadata").await.expect("success");
        connection.assert_requests_match(&[]);
        assert_eq!(resp1, "test-imds-output1");
        assert_eq!(resp2, "test-imds-output2");
    }

    /// Tokens are refreshed up to 120 seconds early to avoid using an expired token.
    #[tokio::test]
    async fn token_refresh_buffer() {
        let connection = TestConnection::new(vec![
            (
                token_request("http://[fd00:ec2::254]", 600),
                token_response(600, TOKEN_A),
            ),
            // t = 0
            (
                imds_request("http://[fd00:ec2::254]/latest/metadata", TOKEN_A),
                imds_response(r#"test-imds-output1"#),
            ),
            // t = 400 (no refresh)
            (
                imds_request("http://[fd00:ec2::254]/latest/metadata", TOKEN_A),
                imds_response(r#"test-imds-output2"#),
            ),
            // t = 550 (within buffer)
            (
                token_request("http://[fd00:ec2::254]", 600),
                token_response(600, TOKEN_B),
            ),
            (
                imds_request("http://[fd00:ec2::254]/latest/metadata", TOKEN_B),
                imds_response(r#"test-imds-output3"#),
            ),
        ]);
        tokio::time::pause();
        let mut time_source = TestingTimeSource::new(UNIX_EPOCH);
        let client = super::Client::builder()
            .configure(
                &ProviderConfig::no_configuration()
                    .with_sleep(TokioSleep::new())
                    .with_http_connector(DynConnector::new(connection.clone()))
                    .with_time_source(TimeSource::testing(&time_source)),
            )
            .endpoint_mode(EndpointMode::IpV6)
            .token_ttl(Duration::from_secs(600))
            .build()
            .await
            .expect("valid client");

        let resp1 = client.get("/latest/metadata").await.expect("success");
        // now the cached credential has expired
        time_source.advance(Duration::from_secs(400));
        let resp2 = client.get("/latest/metadata").await.expect("success");
        time_source.advance(Duration::from_secs(150));
        let resp3 = client.get("/latest/metadata").await.expect("success");
        connection.assert_requests_match(&[]);
        assert_eq!(resp1, "test-imds-output1");
        assert_eq!(resp2, "test-imds-output2");
        assert_eq!(resp3, "test-imds-output3");
    }

    /// 500 error during the GET should be retried
    #[tokio::test]
    #[traced_test]
    async fn retry_500() {
        let connection = TestConnection::new(vec![
            (
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A),
            ),
            (
                imds_request("http://169.254.169.254/latest/metadata", TOKEN_A),
                http::Response::builder().status(500).body("").unwrap(),
            ),
            (
                imds_request("http://169.254.169.254/latest/metadata", TOKEN_A),
                imds_response("ok"),
            ),
        ]);
        let client = make_client(&connection).await;
        assert_eq!(client.get("/latest/metadata").await.expect("success"), "ok");
        connection.assert_requests_match(&[]);

        // all requests should have a user agent header
        for request in connection.requests().iter() {
            assert!(request.actual.headers().get(USER_AGENT).is_some());
        }
    }

    /// 500 error during token acquisition should be retried
    #[tokio::test]
    #[traced_test]
    async fn retry_token_failure() {
        let connection = TestConnection::new(vec![
            (
                token_request("http://169.254.169.254", 21600),
                http::Response::builder().status(500).body("").unwrap(),
            ),
            (
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A),
            ),
            (
                imds_request("http://169.254.169.254/latest/metadata", TOKEN_A),
                imds_response("ok"),
            ),
        ]);
        let client = make_client(&connection).await;
        assert_eq!(client.get("/latest/metadata").await.expect("success"), "ok");
        connection.assert_requests_match(&[]);
    }

    /// 401 error during metadata retrieval must be retried
    #[tokio::test]
    #[traced_test]
    async fn retry_metadata_401() {
        let connection = TestConnection::new(vec![
            (
                token_request("http://169.254.169.254", 21600),
                token_response(0, TOKEN_A),
            ),
            (
                imds_request("http://169.254.169.254/latest/metadata", TOKEN_A),
                http::Response::builder().status(401).body("").unwrap(),
            ),
            (
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_B),
            ),
            (
                imds_request("http://169.254.169.254/latest/metadata", TOKEN_B),
                imds_response("ok"),
            ),
        ]);
        let client = make_client(&connection).await;
        assert_eq!(client.get("/latest/metadata").await.expect("success"), "ok");
        connection.assert_requests_match(&[]);
    }

    /// 403 responses from IMDS during token acquisition MUST NOT be retried
    #[tokio::test]
    #[traced_test]
    async fn no_403_retry() {
        let connection = TestConnection::new(vec![(
            token_request("http://169.254.169.254", 21600),
            http::Response::builder().status(403).body("").unwrap(),
        )]);
        let client = make_client(&connection).await;
        let err = client.get("/latest/metadata").await.expect_err("no token");
        assert_full_error_contains!(err, "forbidden");
        connection.assert_requests_match(&[]);
    }

    /// Successful responses should classify as `RetryKind::Unnecessary`
    #[test]
    fn successful_response_properly_classified() {
        use aws_smithy_http::retry::ClassifyRetry;

        let classifier = ImdsResponseRetryClassifier;
        fn response_200() -> operation::Response {
            operation::Response::new(imds_response("").map(|_| SdkBody::empty()))
        }
        let success = SdkSuccess {
            raw: response_200(),
            parsed: (),
        };
        assert_eq!(
            RetryKind::Unnecessary,
            classifier.classify_retry(Ok::<_, &SdkError<()>>(&success))
        );

        // Emulate a failure to parse the response body (using an io error since it's easy to construct in a test)
        let failure = SdkError::<()>::response_error(
            io::Error::new(io::ErrorKind::BrokenPipe, "fail to parse"),
            response_200(),
        );
        assert_eq!(
            RetryKind::UnretryableFailure,
            classifier.classify_retry(Err::<&SdkSuccess<()>, _>(&failure))
        );
    }

    // since tokens are sent as headers, the tokens need to be valid header values
    #[tokio::test]
    async fn invalid_token() {
        let connection = TestConnection::new(vec![(
            token_request("http://169.254.169.254", 21600),
            token_response(21600, "replaced").map(|_| vec![1, 0]),
        )]);
        let client = make_client(&connection).await;
        let err = client.get("/latest/metadata").await.expect_err("no token");
        assert_full_error_contains!(err, "invalid token");
        connection.assert_requests_match(&[]);
    }

    #[tokio::test]
    async fn non_utf8_response() {
        let connection = TestConnection::new(vec![
            (
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A).map(SdkBody::from),
            ),
            (
                imds_request("http://169.254.169.254/latest/metadata", TOKEN_A),
                http::Response::builder()
                    .status(200)
                    .body(SdkBody::from(vec![0xA0, 0xA1]))
                    .unwrap(),
            ),
        ]);
        let client = make_client(&connection).await;
        let err = client.get("/latest/metadata").await.expect_err("no token");
        assert_full_error_contains!(err, "invalid UTF-8");
        connection.assert_requests_match(&[]);
    }

    /// Verify that the end-to-end real client has a 1-second connect timeout
    #[tokio::test]
    #[cfg(any(feature = "rustls", feature = "native-tls"))]
    async fn one_second_connect_timeout() {
        use crate::imds::client::ImdsError;
        use aws_smithy_types::error::display::DisplayErrorContext;
        use std::time::SystemTime;

        let client = Client::builder()
            // 240.* can never be resolved
            .endpoint(Uri::from_static("http://240.0.0.0"))
            .build()
            .await
            .expect("valid client");
        let now = SystemTime::now();
        let resp = client
            .get("/latest/metadata")
            .await
            .expect_err("240.0.0.0 will never resolve");
        let time_elapsed = now.elapsed().unwrap();
        assert!(
            time_elapsed > Duration::from_secs(1),
            "time_elapsed should be greater than 1s but was {:?}",
            time_elapsed
        );
        assert!(
            time_elapsed < Duration::from_secs(2),
            "time_elapsed should be less than 2s but was {:?}",
            time_elapsed
        );
        match resp {
            err @ ImdsError::FailedToLoadToken(_)
                if format!("{}", DisplayErrorContext(&err)).contains("timeout") => {} // ok,
            other => panic!(
                "wrong error, expected construction failure with TimedOutError inside: {}",
                other
            ),
        }
    }

    #[derive(Debug, Deserialize)]
    struct ImdsConfigTest {
        env: HashMap<String, String>,
        fs: HashMap<String, String>,
        endpoint_override: Option<String>,
        mode_override: Option<String>,
        result: Result<String, String>,
        docs: String,
    }

    #[tokio::test]
    async fn config_tests() -> Result<(), Box<dyn Error>> {
        let test_cases = std::fs::read_to_string("test-data/imds-config/imds-tests.json")?;
        #[derive(Deserialize)]
        struct TestCases {
            tests: Vec<ImdsConfigTest>,
        }

        let test_cases: TestCases = serde_json::from_str(&test_cases)?;
        let test_cases = test_cases.tests;
        for test in test_cases {
            check(test).await;
        }
        Ok(())
    }

    async fn check(test_case: ImdsConfigTest) {
        let (server, watcher) = capture_request(None);
        let provider_config = ProviderConfig::no_configuration()
            .with_sleep(TokioSleep::new())
            .with_env(Env::from(test_case.env))
            .with_fs(Fs::from_map(test_case.fs))
            .with_http_connector(DynConnector::new(server));
        let mut imds_client = Client::builder().configure(&provider_config);
        if let Some(endpoint_override) = test_case.endpoint_override {
            imds_client = imds_client.endpoint(endpoint_override.parse::<Uri>().unwrap());
        }

        if let Some(mode_override) = test_case.mode_override {
            imds_client = imds_client.endpoint_mode(mode_override.parse().unwrap());
        }

        let imds_client = imds_client.build().await;
        let (uri, imds_client) = match (&test_case.result, imds_client) {
            (Ok(uri), Ok(client)) => (uri, client),
            (Err(test), Ok(_client)) => panic!(
                "test should fail: {} but a valid client was made. {}",
                test, test_case.docs
            ),
            (Err(substr), Err(err)) => {
                assert_full_error_contains!(err, substr);
                return;
            }
            (Ok(_uri), Err(e)) => panic!(
                "a valid client should be made but: {}. {}",
                e, test_case.docs
            ),
        };
        // this request will fail, we just want to capture the endpoint configuration
        let _ = imds_client.get("/hello").await;
        assert_eq!(&watcher.expect_request().uri().to_string(), uri);
    }
}

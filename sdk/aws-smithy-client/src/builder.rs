/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::{bounds, erase, retry, Client};
use aws_smithy_async::rt::sleep::{default_async_sleep, SharedAsyncSleep};
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::ConnectorError;
use aws_smithy_types::retry::ReconnectMode;
use aws_smithy_types::timeout::{OperationTimeoutConfig, TimeoutConfig};

#[derive(Clone, Debug)]
struct MaybeRequiresSleep<I> {
    requires_sleep: bool,
    implementation: I,
}

impl<I> MaybeRequiresSleep<I> {
    fn new(requires_sleep: bool, implementation: I) -> Self {
        Self {
            requires_sleep,
            implementation,
        }
    }
}

/// A builder that provides more customization options when constructing a [`Client`].
///
/// To start, call [`Builder::new`]. Then, chain the method calls to configure the `Builder`.
/// When configured to your liking, call [`Builder::build`]. The individual methods have additional
/// documentation.
#[derive(Clone, Debug)]
pub struct Builder<C = (), M = (), R = retry::Standard> {
    connector: MaybeRequiresSleep<C>,
    middleware: M,
    retry_policy: MaybeRequiresSleep<R>,
    operation_timeout_config: Option<OperationTimeoutConfig>,
    sleep_impl: Option<SharedAsyncSleep>,
    reconnect_mode: Option<ReconnectMode>,
}

/// transitional default: disable this behavior by default
const fn default_reconnect_mode() -> ReconnectMode {
    ReconnectMode::ReuseAllConnections
}

impl<C, M> Default for Builder<C, M>
where
    C: Default,
    M: Default,
{
    fn default() -> Self {
        let default_retry_config = retry::Config::default();
        Self {
            connector: MaybeRequiresSleep::new(false, Default::default()),
            middleware: Default::default(),
            retry_policy: MaybeRequiresSleep::new(
                default_retry_config.has_retry(),
                retry::Standard::new(default_retry_config),
            ),
            operation_timeout_config: None,
            sleep_impl: default_async_sleep(),
            reconnect_mode: Some(default_reconnect_mode()),
        }
    }
}

// It'd be nice to include R where R: Default here, but then the caller ends up always having to
// specify R explicitly since type parameter defaults (like the one for R) aren't picked up when R
// cannot be inferred. This is, arguably, a compiler bug/missing language feature, but is
// complicated: https://github.com/rust-lang/rust/issues/27336.
//
// For the time being, we stick with just <C, M> for ::new. Those can usually be inferred since we
// only implement .constructor and .middleware when C and M are () respectively. Users who really
// need a builder for a custom R can use ::default instead.
impl<C, M> Builder<C, M>
where
    C: Default,
    M: Default,
{
    /// Construct a new builder. This does not specify a [connector](Builder::connector)
    /// or [middleware](Builder::middleware).
    /// It uses the [standard retry mechanism](retry::Standard).
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "rustls")]
use crate::erase::DynConnector;
#[cfg(feature = "rustls")]
use crate::http_connector::ConnectorSettings;
#[cfg(feature = "rustls")]
use crate::hyper_ext::Adapter as HyperAdapter;

#[cfg(all(feature = "native-tls", not(feature = "allow-compilation")))]
compile_error!("Feature native-tls has been removed. For upgrade instructions, see: https://awslabs.github.io/smithy-rs/design/transport/connector.html");

/// Max idle connections is not standardized across SDKs. Java V1 and V2 use 50, and Go V2 uses 100.
/// The number below was chosen arbitrarily between those two reference points, and should allow
/// for 14 separate SDK clients in a Lambda where the max file handles is 1024.
#[cfg(feature = "rustls")]
const DEFAULT_MAX_IDLE_CONNECTIONS: usize = 70;

/// Returns default HTTP client settings for hyper.
#[cfg(feature = "rustls")]
fn default_hyper_builder() -> hyper::client::Builder {
    let mut builder = hyper::client::Builder::default();
    builder.pool_max_idle_per_host(DEFAULT_MAX_IDLE_CONNECTIONS);
    builder
}

#[cfg(feature = "rustls")]
impl<M, R> Builder<(), M, R> {
    /// Connect to the service over HTTPS using Rustls using dynamic dispatch.
    pub fn rustls_connector(
        self,
        connector_settings: ConnectorSettings,
    ) -> Builder<DynConnector, M, R> {
        self.connector(DynConnector::new(
            HyperAdapter::builder()
                .hyper_builder(default_hyper_builder())
                .connector_settings(connector_settings)
                .build(crate::conns::https()),
        ))
    }
}

#[cfg(feature = "rustls")]
impl<M, R> Builder<(), M, R> {
    /// Create a Smithy client builder with an HTTPS connector and the [standard retry
    /// policy](crate::retry::Standard) over the default middleware implementation.
    ///
    /// For convenience, this constructor type-erases the concrete TLS connector backend used using
    /// dynamic dispatch. This comes at a slight runtime performance cost. See
    /// [`DynConnector`](crate::erase::DynConnector) for details. To avoid that overhead, use
    /// [`Builder::rustls_connector`] instead.
    #[cfg(feature = "rustls")]
    pub fn dyn_https_connector(
        self,
        connector_settings: ConnectorSettings,
    ) -> Builder<DynConnector, M, R> {
        let with_https = |b: Builder<_, M, R>| b.rustls_connector(connector_settings);
        with_https(self)
    }
}

impl<M, R> Builder<(), M, R> {
    /// Specify the connector for the eventual client to use.
    ///
    /// The connector dictates how requests are turned into responses. Normally, this would entail
    /// sending the request to some kind of remote server, but in certain settings it's useful to
    /// be able to use a custom connector instead, such as to mock the network for tests.
    ///
    /// If you just want to specify a function from request to response instead, use
    /// [`Builder::connector_fn`].
    pub fn connector<C>(self, connector: C) -> Builder<C, M, R> {
        Builder {
            connector: MaybeRequiresSleep::new(false, connector),
            middleware: self.middleware,
            retry_policy: self.retry_policy,
            operation_timeout_config: self.operation_timeout_config,
            sleep_impl: self.sleep_impl,
            reconnect_mode: self.reconnect_mode,
        }
    }

    /// Use a function that directly maps each request to a response as a connector.
    ///
    /// ```no_run
    /// use aws_smithy_client::Builder;
    /// use aws_smithy_http::body::SdkBody;
    /// let client = Builder::new()
    /// # /*
    ///   .middleware(..)
    /// # */
    /// # .middleware(tower::layer::util::Identity::new())
    ///   .connector_fn(|req: http::Request<SdkBody>| {
    ///     async move {
    ///       Ok(http::Response::new(SdkBody::empty()))
    ///     }
    ///   })
    ///   .build();
    /// # client.check();
    /// ```
    pub fn connector_fn<F, FF>(self, map: F) -> Builder<tower::util::ServiceFn<F>, M, R>
    where
        F: Fn(http::Request<SdkBody>) -> FF + Send,
        FF: std::future::Future<Output = Result<http::Response<SdkBody>, ConnectorError>>,
        // NOTE: The extra bound here is to help the type checker give better errors earlier.
        tower::util::ServiceFn<F>: bounds::SmithyConnector,
    {
        self.connector(tower::service_fn(map))
    }
}

impl<C, R> Builder<C, (), R> {
    /// Specify the middleware for the eventual client ot use.
    ///
    /// The middleware adjusts requests before they are dispatched to the connector. It is
    /// responsible for filling in any request parameters that aren't specified by the Smithy
    /// protocol definition, such as those used for routing (like the URL), authentication, and
    /// authorization.
    ///
    /// The middleware takes the form of a [`tower::Layer`] that wraps the actual connection for
    /// each request. The [`tower::Service`] that the middleware produces must accept requests of
    /// the type [`aws_smithy_http::operation::Request`] and return responses of the type
    /// [`http::Response<SdkBody>`], most likely by modifying the provided request in place,
    /// passing it to the inner service, and then ultimately returning the inner service's
    /// response.
    ///
    /// If your requests are already ready to be sent and need no adjustment, you can use
    /// [`tower::layer::util::Identity`] as your middleware.
    pub fn middleware<M>(self, middleware: M) -> Builder<C, M, R> {
        Builder {
            connector: self.connector,
            retry_policy: self.retry_policy,
            operation_timeout_config: self.operation_timeout_config,
            middleware,
            sleep_impl: self.sleep_impl,
            reconnect_mode: self.reconnect_mode,
        }
    }

    /// Use a function-like middleware that directly maps each request.
    ///
    /// ```no_run
    /// use aws_smithy_client::Builder;
    /// use aws_smithy_client::erase::DynConnector;
    /// use aws_smithy_client::never::NeverConnector;
    /// use aws_smithy_http::body::SdkBody;
    /// let my_connector = DynConnector::new(
    ///     // Your own connector here or use `dyn_https_connector()`
    ///     # NeverConnector::new()
    /// );
    /// let client = Builder::new()
    ///   .connector(my_connector)
    ///   .middleware_fn(|req: aws_smithy_http::operation::Request| {
    ///     req
    ///   })
    ///   .build();
    /// # client.check();
    /// ```
    pub fn middleware_fn<F>(self, map: F) -> Builder<C, tower::util::MapRequestLayer<F>, R>
    where
        F: Fn(aws_smithy_http::operation::Request) -> aws_smithy_http::operation::Request
            + Clone
            + Send
            + Sync
            + 'static,
    {
        self.middleware(tower::util::MapRequestLayer::new(map))
    }
}

impl<C, M> Builder<C, M, retry::Standard> {
    /// Specify the retry policy for the eventual client to use.
    ///
    /// By default, the Smithy client uses a standard retry policy that works well in most
    /// settings. You can use this method to override that policy with a custom one. A new policy
    /// instance will be instantiated for each request using [`retry::NewRequestPolicy`]. Each
    /// policy instance must implement [`tower::retry::Policy`].
    ///
    /// If you just want to modify the policy _configuration_ for the standard retry policy, use
    /// [`Builder::set_retry_config`].
    pub fn retry_policy<R>(self, retry_policy: R) -> Builder<C, M, R> {
        Builder {
            connector: self.connector,
            retry_policy: MaybeRequiresSleep::new(false, retry_policy),
            operation_timeout_config: self.operation_timeout_config,
            middleware: self.middleware,
            sleep_impl: self.sleep_impl,
            reconnect_mode: self.reconnect_mode,
        }
    }
}

impl<C, M> Builder<C, M> {
    /// Set the standard retry policy's configuration. When `config` is `None`,
    /// the default retry policy will be used.
    pub fn set_retry_config(&mut self, config: Option<retry::Config>) -> &mut Self {
        let config = config.unwrap_or_default();
        self.retry_policy =
            MaybeRequiresSleep::new(config.has_retry(), retry::Standard::new(config));
        self
    }

    /// Set the standard retry policy's configuration.
    pub fn retry_config(mut self, config: retry::Config) -> Self {
        self.set_retry_config(Some(config));
        self
    }

    /// Set operation timeout config for the client. If `operation_timeout_config` is
    /// `None`, timeouts will be disabled.
    pub fn set_operation_timeout_config(
        &mut self,
        operation_timeout_config: Option<OperationTimeoutConfig>,
    ) -> &mut Self {
        self.operation_timeout_config = operation_timeout_config;
        self
    }

    /// Set operation timeout config for the client.
    pub fn operation_timeout_config(
        mut self,
        operation_timeout_config: OperationTimeoutConfig,
    ) -> Self {
        self.operation_timeout_config = Some(operation_timeout_config);
        self
    }

    /// Set [`aws_smithy_async::rt::sleep::SharedAsyncSleep`] that the [`Client`] will use to create things like timeout futures.
    pub fn set_sleep_impl(&mut self, async_sleep: Option<SharedAsyncSleep>) -> &mut Self {
        self.sleep_impl = async_sleep;
        self
    }

    /// Set [`aws_smithy_async::rt::sleep::SharedAsyncSleep`] that the [`Client`] will use to create things like timeout futures.
    pub fn sleep_impl(mut self, async_sleep: SharedAsyncSleep) -> Self {
        self.set_sleep_impl(Some(async_sleep));
        self
    }
}

impl<C, M, R> Builder<C, M, R> {
    /// Use a connector that wraps the current connector.
    pub fn map_connector<F, C2>(self, map: F) -> Builder<C2, M, R>
    where
        F: FnOnce(C) -> C2,
    {
        Builder {
            connector: MaybeRequiresSleep::new(
                self.connector.requires_sleep,
                map(self.connector.implementation),
            ),
            middleware: self.middleware,
            retry_policy: self.retry_policy,
            operation_timeout_config: self.operation_timeout_config,
            sleep_impl: self.sleep_impl,
            reconnect_mode: self.reconnect_mode,
        }
    }

    /// Use a middleware that wraps the current middleware.
    pub fn map_middleware<F, M2>(self, map: F) -> Builder<C, M2, R>
    where
        F: FnOnce(M) -> M2,
    {
        Builder {
            connector: self.connector,
            middleware: map(self.middleware),
            retry_policy: self.retry_policy,
            operation_timeout_config: self.operation_timeout_config,
            sleep_impl: self.sleep_impl,
            reconnect_mode: self.reconnect_mode,
        }
    }

    /// Set the [`ReconnectMode`] for the retry strategy
    ///
    /// By default, no reconnection occurs.
    ///
    /// When enabled and a transient error is encountered, the connection in use will be poisoned.
    /// This prevents reusing a connection to a potentially bad host.
    pub fn reconnect_mode(mut self, reconnect_mode: ReconnectMode) -> Self {
        self.set_reconnect_mode(Some(reconnect_mode));
        self
    }

    /// Set the [`ReconnectMode`] for the retry strategy
    ///
    /// By default, no reconnection occurs.
    ///
    /// When enabled and a transient error is encountered, the connection in use will be poisoned.
    /// This prevents reusing a connection to a potentially bad host.
    pub fn set_reconnect_mode(&mut self, reconnect_mode: Option<ReconnectMode>) -> &mut Self {
        self.reconnect_mode = reconnect_mode;
        self
    }

    /// Enable reconnection on transient errors
    ///
    /// By default, when a transient error is encountered, the connection in use will be poisoned.
    /// This prevents reusing a connection to a potentially bad host but may increase the load on
    /// the server.
    pub fn reconnect_on_transient_errors(self) -> Self {
        self.reconnect_mode(ReconnectMode::ReconnectOnTransientError)
    }

    /// Build a Smithy service [`Client`].
    pub fn build(self) -> Client<C, M, R> {
        let operation_timeout_config = self
            .operation_timeout_config
            .unwrap_or_else(|| TimeoutConfig::disabled().into());
        if self.sleep_impl.is_none() {
            const ADDITIONAL_HELP: &str =
                "Either disable retry by setting max attempts to one, or pass in a `sleep_impl`. \
                If you're not using Tokio, then an implementation of the `AsyncSleep` trait from \
                the `aws-smithy-async` crate is required for your async runtime. If you are using \
                Tokio, then make sure the `rt-tokio` feature is enabled to have its sleep \
                implementation set automatically.";
            if self.connector.requires_sleep {
                panic!("Socket-level retries for the default connector require a `sleep_impl`, but none was passed into the builder. {ADDITIONAL_HELP}");
            }
            if self.retry_policy.requires_sleep {
                panic!("Retries require a `sleep_impl`, but none was passed into the builder. {ADDITIONAL_HELP}");
            }
            if operation_timeout_config.has_timeouts() {
                panic!("Operation timeouts require a `sleep_impl`, but none was passed into the builder. {ADDITIONAL_HELP}");
            }
        }
        Client {
            connector: self.connector.implementation,
            retry_policy: self.retry_policy.implementation,
            middleware: self.middleware,
            operation_timeout_config,
            sleep_impl: self.sleep_impl,
            reconnect_mode: self.reconnect_mode.unwrap_or(default_reconnect_mode()),
        }
    }
}

impl<C, M, R> Builder<C, M, R>
where
    C: bounds::SmithyConnector,
    M: bounds::SmithyMiddleware<erase::DynConnector> + Send + Sync + 'static,
    R: retry::NewRequestPolicy,
{
    /// Build a type-erased Smithy service [`Client`].
    ///
    /// Note that if you're using the standard retry mechanism, [`retry::Standard`], `DynClient<R>`
    /// is equivalent to [`Client`] with no type arguments.
    ///
    /// ```no_run
    /// # #[cfg(feature = "https")]
    /// # fn not_main() {
    /// use aws_smithy_client::{Builder, Client};
    /// struct MyClient {
    ///     client: aws_smithy_client::Client,
    /// }
    ///
    /// let client = Builder::new()
    ///     .https()
    ///     .middleware(tower::layer::util::Identity::new())
    ///     .build_dyn();
    /// let client = MyClient { client };
    /// # client.client.check();
    /// # }
    pub fn build_dyn(self) -> erase::DynClient<R> {
        self.build().into_dyn()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::never::NeverConnector;
    use aws_smithy_async::rt::sleep::{AsyncSleep, Sleep};
    use std::panic::{self, AssertUnwindSafe};
    use std::time::Duration;

    #[derive(Clone, Debug)]
    struct StubSleep;
    impl AsyncSleep for StubSleep {
        fn sleep(&self, _duration: Duration) -> Sleep {
            todo!()
        }
    }

    #[test]
    fn defaults_dont_panic() {
        let builder = Builder::new()
            .connector(NeverConnector::new())
            .middleware(tower::layer::util::Identity::new());

        let _ = builder.build();
    }

    #[test]
    fn defaults_panic_if_default_tokio_sleep_not_available() {
        let mut builder = Builder::new()
            .connector(NeverConnector::new())
            .middleware(tower::layer::util::Identity::new());
        builder.set_sleep_impl(None);

        let result = panic::catch_unwind(AssertUnwindSafe(move || {
            let _ = builder.build();
        }));
        assert!(result.is_err());
    }

    #[test]
    fn timeouts_without_sleep_panics() {
        let mut builder = Builder::new()
            .connector(NeverConnector::new())
            .middleware(tower::layer::util::Identity::new());
        builder.set_sleep_impl(None);

        let timeout_config = TimeoutConfig::builder()
            .connect_timeout(Duration::from_secs(1))
            .build();
        assert!(timeout_config.has_timeouts());
        builder.set_operation_timeout_config(Some(timeout_config.into()));

        let result = panic::catch_unwind(AssertUnwindSafe(move || {
            let _ = builder.build();
        }));
        assert!(result.is_err());
    }

    #[test]
    fn retry_without_sleep_panics() {
        let mut builder = Builder::new()
            .connector(NeverConnector::new())
            .middleware(tower::layer::util::Identity::new());
        builder.set_sleep_impl(None);

        let retry_config = retry::Config::default();
        assert!(retry_config.has_retry());
        builder.set_retry_config(Some(retry_config));

        let result = panic::catch_unwind(AssertUnwindSafe(move || {
            let _ = builder.build();
        }));
        assert!(result.is_err());
    }

    #[test]
    fn custom_retry_policy_without_sleep_doesnt_panic() {
        let mut builder = Builder::new()
            .connector(NeverConnector::new())
            .middleware(tower::layer::util::Identity::new())
            // Using standard retry here as a shortcut in the test; someone setting
            // a custom retry policy would manually implement the required traits
            .retry_policy(retry::Standard::default());
        builder.set_sleep_impl(None);
        let _ = builder.build();
    }

    #[test]
    fn no_panics_when_sleep_given() {
        let mut builder = Builder::new()
            .connector(NeverConnector::new())
            .middleware(tower::layer::util::Identity::new());

        let timeout_config = TimeoutConfig::builder()
            .connect_timeout(Duration::from_secs(1))
            .build();
        assert!(timeout_config.has_timeouts());
        builder.set_operation_timeout_config(Some(timeout_config.into()));

        let retry_config = retry::Config::default();
        assert!(retry_config.has_retry());
        builder.set_retry_config(Some(retry_config));

        let _ = builder.build();
    }

    #[test]
    fn builder_connection_helpers_are_dyn() {
        #[cfg(feature = "rustls")]
        let _builder: Builder<DynConnector, (), _> =
            Builder::new().rustls_connector(Default::default());
    }
}

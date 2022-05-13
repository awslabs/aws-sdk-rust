/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::sync::Arc;

use crate::{bounds, erase, retry, Client, TriState, MISSING_SLEEP_IMPL_RECOMMENDATION};
use aws_smithy_async::rt::sleep::{default_async_sleep, AsyncSleep};
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::ConnectorError;
use aws_smithy_types::timeout;

/// A builder that provides more customization options when constructing a [`Client`].
///
/// To start, call [`Builder::new`]. Then, chain the method calls to configure the `Builder`.
/// When configured to your liking, call [`Builder::build`]. The individual methods have additional
/// documentation.
#[derive(Clone, Debug, Default)]
pub struct Builder<C = (), M = (), R = retry::Standard> {
    connector: C,
    middleware: M,
    retry_policy: R,
    timeout_config: timeout::Config,
    sleep_impl: TriState<Arc<dyn AsyncSleep>>,
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
            connector,
            retry_policy: self.retry_policy,
            middleware: self.middleware,
            timeout_config: self.timeout_config,
            sleep_impl: self.sleep_impl,
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
            timeout_config: self.timeout_config,
            middleware,
            sleep_impl: self.sleep_impl,
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
    ///     // Your own connector here or use `dyn_https()`
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
            retry_policy,
            timeout_config: self.timeout_config,
            middleware: self.middleware,
            sleep_impl: self.sleep_impl,
        }
    }
}

impl<C, M> Builder<C, M> {
    /// Set the standard retry policy's configuration.
    pub fn set_retry_config(&mut self, config: retry::Config) {
        self.retry_policy.with_config(config);
    }

    /// Set a timeout config for the builder
    pub fn set_timeout_config(&mut self, timeout_config: timeout::Config) {
        self.timeout_config = timeout_config;
    }

    /// Set the [`AsyncSleep`] function that the [`Client`] will use to create things like timeout futures.
    pub fn set_sleep_impl(&mut self, async_sleep: Option<Arc<dyn AsyncSleep>>) {
        self.sleep_impl = async_sleep.into();
    }

    /// Set the [`AsyncSleep`] function that the [`Client`] will use to create things like timeout futures.
    pub fn sleep_impl(mut self, async_sleep: Option<Arc<dyn AsyncSleep>>) -> Self {
        self.set_sleep_impl(async_sleep);
        self
    }

    /// Sets the sleep implementation to [`default_async_sleep`].
    pub fn default_async_sleep(mut self) -> Self {
        self.sleep_impl = TriState::or_unset(default_async_sleep());
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
            connector: map(self.connector),
            middleware: self.middleware,
            retry_policy: self.retry_policy,
            timeout_config: self.timeout_config,
            sleep_impl: self.sleep_impl,
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
            timeout_config: self.timeout_config,
            sleep_impl: self.sleep_impl,
        }
    }

    /// Build a Smithy service [`Client`].
    pub fn build(self) -> Client<C, M, R> {
        if matches!(self.sleep_impl, TriState::Unset) {
            if self.timeout_config.has_timeouts() {
                tracing::warn!(
                    "One or more timeouts were set, but no `sleep_impl` was passed into the \
                    builder. Timeouts and retry both require a sleep implementation. No timeouts \
                    will occur with the current configuration. {}",
                    MISSING_SLEEP_IMPL_RECOMMENDATION
                );
            } else {
                tracing::warn!(
                    "Retries require a `sleep_impl`, but none was passed into the builder. \
                    No retries will occur with the current configuration. {}",
                    MISSING_SLEEP_IMPL_RECOMMENDATION
                );
            }
        }

        Client {
            connector: self.connector,
            retry_policy: self.retry_policy,
            middleware: self.middleware,
            timeout_config: self.timeout_config,
            sleep_impl: self.sleep_impl,
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
    use aws_smithy_async::rt::sleep::Sleep;
    use aws_smithy_types::timeout;
    use aws_smithy_types::tristate::TriState;
    use std::time::Duration;

    #[derive(Clone, Debug)]
    struct StubSleep;
    impl AsyncSleep for StubSleep {
        fn sleep(&self, _duration: Duration) -> Sleep {
            todo!()
        }
    }

    const TIMEOUTS_WITHOUT_SLEEP_MSG: &str =
        "One or more timeouts were set, but no `sleep_impl` was passed into the builder";
    const RETRIES_WITHOUT_SLEEP_MSG: &str =
        "Retries require a `sleep_impl`, but none was passed into the builder.";
    const RECOMMENDATION_MSG: &str =
        "consider using the `aws-config` crate to load a shared config";

    #[test]
    #[tracing_test::traced_test]
    fn sleep_impl_given_no_warns() {
        let _client = Builder::new()
            .connector(NeverConnector::new())
            .middleware(tower::layer::util::Identity::new())
            .sleep_impl(Some(Arc::new(StubSleep)))
            .build();

        assert!(!logs_contain(TIMEOUTS_WITHOUT_SLEEP_MSG));
        assert!(!logs_contain(RETRIES_WITHOUT_SLEEP_MSG));
        assert!(!logs_contain(RECOMMENDATION_MSG));
    }

    #[test]
    #[tracing_test::traced_test]
    fn timeout_missing_sleep_impl_warn() {
        let mut builder = Builder::new()
            .connector(NeverConnector::new())
            .middleware(tower::layer::util::Identity::new());
        let http_timeout_config =
            timeout::Http::new().with_connect_timeout(TriState::Set(Duration::from_secs(1)));
        let timeout_config = timeout::Config::new().with_http_timeouts(http_timeout_config);
        builder.set_timeout_config(timeout_config);
        builder.build();

        assert!(logs_contain(TIMEOUTS_WITHOUT_SLEEP_MSG));
        assert!(!logs_contain(RETRIES_WITHOUT_SLEEP_MSG));
        assert!(logs_contain(RECOMMENDATION_MSG));
    }

    #[test]
    #[tracing_test::traced_test]
    fn retry_missing_sleep_impl_warn() {
        Builder::new()
            .connector(NeverConnector::new())
            .middleware(tower::layer::util::Identity::new())
            .build();

        assert!(!logs_contain(TIMEOUTS_WITHOUT_SLEEP_MSG));
        assert!(logs_contain(RETRIES_WITHOUT_SLEEP_MSG));
        assert!(logs_contain(RECOMMENDATION_MSG));
    }
}

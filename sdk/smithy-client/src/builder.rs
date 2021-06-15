use crate::{bounds, erase, retry, BoxError, Client};
use smithy_http::body::SdkBody;

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
    /// Construct a new builder.
    ///
    /// This will
    ///
    /// You will likely want to , as it does not specify a [connector](Builder::connector)
    /// or [middleware](Builder::middleware). It uses the [standard retry
    /// mechanism](retry::Standard).
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
    /// If you want to use a custom hyper connector, use [`Builder::hyper`].
    ///
    /// If you just want to specify a function from request to response instead, use
    /// [`Builder::map_connector`].
    pub fn connector<C>(self, connector: C) -> Builder<C, M, R> {
        Builder {
            connector,
            retry_policy: self.retry_policy,
            middleware: self.middleware,
        }
    }

    /// Use a function that directly maps each request to a response as a connector.
    ///
    /// ```rust
    /// use smithy_client::Builder;
    /// use smithy_http::body::SdkBody;
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
        FF: std::future::Future<Output = Result<http::Response<SdkBody>, BoxError>>,
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
    /// the type [`smithy_http::operation::Request`] and return responses of the type
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
            middleware,
        }
    }

    /// Use a function-like middleware that directly maps each request.
    ///
    /// ```rust
    /// use smithy_client::Builder;
    /// use smithy_http::body::SdkBody;
    /// let client = Builder::new()
    ///   .https()
    ///   .middleware_fn(|req: smithy_http::operation::Request| {
    ///     req
    ///   })
    ///   .build();
    /// # client.check();
    /// ```
    pub fn middleware_fn<F>(self, map: F) -> Builder<C, tower::util::MapRequestLayer<F>, R>
    where
        F: Fn(smithy_http::operation::Request) -> smithy_http::operation::Request
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
            middleware: self.middleware,
        }
    }
}

impl<C, M> Builder<C, M> {
    /// Set the standard retry policy's configuration.
    pub fn set_retry_config(&mut self, config: retry::Config) {
        self.retry_policy.with_config(config);
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
        }
    }

    /// Build a Smithy service [`Client`].
    pub fn build(self) -> Client<C, M, R> {
        Client {
            connector: self.connector,
            retry_policy: self.retry_policy,
            middleware: self.middleware,
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
    /// ```rust
    /// # #[cfg(feature = "https")]
    /// # fn not_main() {
    /// use smithy_client::{Builder, Client};
    /// struct MyClient {
    ///     client: smithy_client::Client,
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

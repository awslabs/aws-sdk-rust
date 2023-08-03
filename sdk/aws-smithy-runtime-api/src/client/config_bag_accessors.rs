/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::auth::{
    AuthOptionResolver, AuthOptionResolverParams, AuthSchemeId, DynAuthOptionResolver,
    SharedHttpAuthScheme,
};
use crate::client::connectors::{Connector, DynConnector};
use crate::client::identity::{
    ConfiguredIdentityResolver, IdentityResolvers, SharedIdentityResolver,
};
use crate::client::orchestrator::{
    DynEndpointResolver, DynResponseDeserializer, EndpointResolver, EndpointResolverParams,
    LoadedRequestBody, ResponseDeserializer, SharedRequestSerializer, NOT_NEEDED,
};
use crate::client::retries::{DynRetryStrategy, RetryClassifiers, RetryStrategy};
use aws_smithy_async::rt::sleep::SharedAsyncSleep;
use aws_smithy_async::time::{SharedTimeSource, TimeSource};
use aws_smithy_types::config_bag::{AppendItemIter, CloneableLayer, ConfigBag, FrozenLayer, Layer};
use std::fmt::Debug;

// Place traits in a private module so that they can be used in the public API without being a part of the public API.
mod internal {
    use aws_smithy_types::config_bag::{
        CloneableLayer, ConfigBag, FrozenLayer, Layer, Storable, Store, StoreAppend, StoreReplace,
    };
    use std::fmt::Debug;

    pub trait Settable {
        fn unset<T: Send + Sync + Clone + Debug + 'static>(&mut self);

        fn store_put<T>(&mut self, value: T)
        where
            T: Storable<Storer = StoreReplace<T>>;

        fn store_append<T>(&mut self, item: T)
        where
            T: Storable<Storer = StoreAppend<T>>;
    }

    impl Settable for Layer {
        fn unset<T: Send + Sync + Clone + Debug + 'static>(&mut self) {
            Layer::unset::<T>(self);
        }

        fn store_put<T>(&mut self, value: T)
        where
            T: Storable<Storer = StoreReplace<T>>,
        {
            Layer::store_put(self, value);
        }

        fn store_append<T>(&mut self, item: T)
        where
            T: Storable<Storer = StoreAppend<T>>,
        {
            Layer::store_append(self, item);
        }
    }

    pub trait CloneableSettable {
        fn store_put<T>(&mut self, value: T)
        where
            T: Storable<Storer = StoreReplace<T>> + Clone;

        fn store_append<T>(&mut self, item: T)
        where
            T: Storable<Storer = StoreAppend<T>> + Clone;
    }

    impl<S> CloneableSettable for S
    where
        S: Settable,
    {
        fn store_put<T>(&mut self, value: T)
        where
            T: Storable<Storer = StoreReplace<T>> + Clone,
        {
            Settable::store_put(self, value);
        }

        fn store_append<T>(&mut self, item: T)
        where
            T: Storable<Storer = StoreAppend<T>> + Clone,
        {
            Settable::store_append(self, item);
        }
    }

    impl CloneableSettable for CloneableLayer {
        fn store_put<T>(&mut self, value: T)
        where
            T: Storable<Storer = StoreReplace<T>> + Clone,
        {
            CloneableLayer::store_put(self, value);
        }

        fn store_append<T>(&mut self, item: T)
        where
            T: Storable<Storer = StoreAppend<T>> + Clone,
        {
            CloneableLayer::store_append(self, item);
        }
    }

    pub trait Gettable {
        fn load<T: Storable>(&self) -> <T::Storer as Store>::ReturnedType<'_>;
    }

    impl Gettable for ConfigBag {
        fn load<T: Storable>(&self) -> <T::Storer as Store>::ReturnedType<'_> {
            ConfigBag::load::<T>(self)
        }
    }

    impl Gettable for CloneableLayer {
        fn load<T: Storable>(&self) -> <T::Storer as Store>::ReturnedType<'_> {
            Layer::load::<T>(self)
        }
    }

    impl Gettable for Layer {
        fn load<T: Storable>(&self) -> <T::Storer as Store>::ReturnedType<'_> {
            Layer::load::<T>(self)
        }
    }

    impl Gettable for FrozenLayer {
        fn load<T: Storable>(&self) -> <T::Storer as Store>::ReturnedType<'_> {
            Layer::load::<T>(self)
        }
    }
}

use internal::{CloneableSettable, Gettable, Settable};

pub trait ConfigBagAccessors {
    fn auth_option_resolver_params(&self) -> &AuthOptionResolverParams
    where
        Self: Gettable,
    {
        self.load::<AuthOptionResolverParams>()
            .expect("auth option resolver params must be set")
    }
    fn set_auth_option_resolver_params(
        &mut self,
        auth_option_resolver_params: AuthOptionResolverParams,
    ) where
        Self: Settable,
    {
        self.store_put::<AuthOptionResolverParams>(auth_option_resolver_params);
    }

    fn auth_option_resolver(&self) -> &dyn AuthOptionResolver
    where
        Self: Gettable,
    {
        self.load::<DynAuthOptionResolver>()
            .expect("an auth option resolver must be set")
    }

    fn set_auth_option_resolver(&mut self, auth_option_resolver: DynAuthOptionResolver)
    where
        Self: Settable,
    {
        self.store_put::<DynAuthOptionResolver>(auth_option_resolver);
    }

    fn endpoint_resolver_params(&self) -> &EndpointResolverParams
    where
        Self: Gettable,
    {
        self.load::<EndpointResolverParams>()
            .expect("endpoint resolver params must be set")
    }

    fn set_endpoint_resolver_params(&mut self, endpoint_resolver_params: EndpointResolverParams)
    where
        Self: Settable,
    {
        self.store_put::<EndpointResolverParams>(endpoint_resolver_params);
    }

    fn endpoint_resolver(&self) -> &dyn EndpointResolver
    where
        Self: Gettable,
    {
        self.load::<DynEndpointResolver>()
            .expect("an endpoint resolver must be set")
    }

    fn set_endpoint_resolver(&mut self, endpoint_resolver: DynEndpointResolver)
    where
        Self: Settable,
    {
        self.store_put::<DynEndpointResolver>(endpoint_resolver);
    }

    /// Returns the configured identity resolvers.
    fn identity_resolvers(&self) -> IdentityResolvers
    where
        Self: Gettable,
    {
        IdentityResolvers::new(self.load::<ConfiguredIdentityResolver>())
    }

    /// Adds an identity resolver to the config.
    fn push_identity_resolver(
        &mut self,
        auth_scheme_id: AuthSchemeId,
        identity_resolver: SharedIdentityResolver,
    ) where
        Self: CloneableSettable,
    {
        self.store_append::<ConfiguredIdentityResolver>(ConfiguredIdentityResolver::new(
            auth_scheme_id,
            identity_resolver,
        ));
    }

    fn connector(&self) -> &dyn Connector
    where
        Self: Gettable,
    {
        self.load::<DynConnector>().expect("missing connector")
    }

    fn set_connector(&mut self, connection: DynConnector)
    where
        Self: Settable,
    {
        self.store_put::<DynConnector>(connection);
    }

    /// Returns the configured HTTP auth schemes.
    fn http_auth_schemes(&self) -> HttpAuthSchemes<'_>
    where
        Self: Gettable,
    {
        HttpAuthSchemes::new(self.load::<SharedHttpAuthScheme>())
    }

    /// Adds a HTTP auth scheme to the config.
    fn push_http_auth_scheme(&mut self, auth_scheme: SharedHttpAuthScheme)
    where
        Self: Settable,
    {
        self.store_append::<SharedHttpAuthScheme>(auth_scheme);
    }

    fn request_serializer(&self) -> SharedRequestSerializer
    where
        Self: Gettable,
    {
        self.load::<SharedRequestSerializer>()
            .expect("missing request serializer")
            .clone()
    }
    fn set_request_serializer(&mut self, request_serializer: SharedRequestSerializer)
    where
        Self: Settable,
    {
        self.store_put::<SharedRequestSerializer>(request_serializer);
    }

    fn response_deserializer(&self) -> &dyn ResponseDeserializer
    where
        Self: Gettable,
    {
        self.load::<DynResponseDeserializer>()
            .expect("missing response deserializer")
    }
    fn set_response_deserializer(&mut self, response_deserializer: DynResponseDeserializer)
    where
        Self: Settable,
    {
        self.store_put::<DynResponseDeserializer>(response_deserializer);
    }

    fn retry_classifiers(&self) -> &RetryClassifiers
    where
        Self: Gettable,
    {
        self.load::<RetryClassifiers>()
            .expect("retry classifiers must be set")
    }
    fn set_retry_classifiers(&mut self, retry_classifiers: RetryClassifiers)
    where
        Self: Settable,
    {
        self.store_put::<RetryClassifiers>(retry_classifiers);
    }

    fn retry_strategy(&self) -> Option<&dyn RetryStrategy>
    where
        Self: Gettable,
    {
        self.load::<DynRetryStrategy>().map(|rs| rs as _)
    }
    fn set_retry_strategy(&mut self, retry_strategy: DynRetryStrategy)
    where
        Self: Settable,
    {
        self.store_put::<DynRetryStrategy>(retry_strategy);
    }

    fn request_time(&self) -> Option<SharedTimeSource>
    where
        Self: Gettable,
    {
        self.load::<SharedTimeSource>().cloned()
    }
    fn set_request_time(&mut self, time_source: impl TimeSource + 'static)
    where
        Self: Settable,
    {
        self.store_put::<SharedTimeSource>(SharedTimeSource::new(time_source));
    }

    fn sleep_impl(&self) -> Option<SharedAsyncSleep>
    where
        Self: Gettable,
    {
        self.load::<SharedAsyncSleep>().cloned()
    }
    fn set_sleep_impl(&mut self, async_sleep: Option<SharedAsyncSleep>)
    where
        Self: Settable,
    {
        if let Some(sleep_impl) = async_sleep {
            self.store_put::<SharedAsyncSleep>(sleep_impl);
        } else {
            self.unset::<SharedAsyncSleep>();
        }
    }

    fn loaded_request_body(&self) -> &LoadedRequestBody
    where
        Self: Gettable,
    {
        self.load::<LoadedRequestBody>().unwrap_or(&NOT_NEEDED)
    }
    fn set_loaded_request_body(&mut self, loaded_request_body: LoadedRequestBody)
    where
        Self: Settable,
    {
        self.store_put::<LoadedRequestBody>(loaded_request_body);
    }
}

impl ConfigBagAccessors for ConfigBag {}
impl ConfigBagAccessors for FrozenLayer {}
impl ConfigBagAccessors for CloneableLayer {}
impl ConfigBagAccessors for Layer {}

/// Accessor for HTTP auth schemes.
#[derive(Debug)]
pub struct HttpAuthSchemes<'a> {
    inner: AppendItemIter<'a, SharedHttpAuthScheme>,
}

impl<'a> HttpAuthSchemes<'a> {
    pub(crate) fn new(inner: AppendItemIter<'a, SharedHttpAuthScheme>) -> Self {
        Self { inner }
    }

    /// Returns the HTTP auth scheme with the given ID, if there is one.
    pub fn scheme(mut self, scheme_id: AuthSchemeId) -> Option<SharedHttpAuthScheme> {
        use crate::client::auth::HttpAuthScheme;
        self.inner
            .find(|&scheme| scheme.scheme_id() == scheme_id)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::auth::{HttpAuthScheme, HttpRequestSigner};
    use crate::client::config_bag_accessors::ConfigBagAccessors;
    use aws_smithy_types::config_bag::{ConfigBag, Layer};

    #[test]
    fn test_shared_http_auth_scheme_configuration() {
        #[derive(Debug)]
        struct TestHttpAuthScheme(&'static str);
        impl HttpAuthScheme for TestHttpAuthScheme {
            fn scheme_id(&self) -> AuthSchemeId {
                AuthSchemeId::new(self.0)
            }

            fn identity_resolver(&self, _: &IdentityResolvers) -> Option<SharedIdentityResolver> {
                unreachable!("this shouldn't get called in this test")
            }

            fn request_signer(&self) -> &dyn HttpRequestSigner {
                unreachable!("this shouldn't get called in this test")
            }
        }

        let mut config_bag = ConfigBag::base();

        let mut layer = Layer::new("first");
        layer.push_http_auth_scheme(SharedHttpAuthScheme::new(TestHttpAuthScheme("scheme_1")));
        config_bag.push_layer(layer);

        let mut layer = Layer::new("second");
        layer.push_http_auth_scheme(SharedHttpAuthScheme::new(TestHttpAuthScheme("scheme_2")));
        layer.push_http_auth_scheme(SharedHttpAuthScheme::new(TestHttpAuthScheme("scheme_3")));
        config_bag.push_layer(layer);

        assert!(config_bag
            .http_auth_schemes()
            .scheme(AuthSchemeId::new("does-not-exist"))
            .is_none());
        assert_eq!(
            AuthSchemeId::new("scheme_1"),
            config_bag
                .http_auth_schemes()
                .scheme(AuthSchemeId::new("scheme_1"))
                .unwrap()
                .scheme_id()
        );
        assert_eq!(
            AuthSchemeId::new("scheme_2"),
            config_bag
                .http_auth_schemes()
                .scheme(AuthSchemeId::new("scheme_2"))
                .unwrap()
                .scheme_id()
        );
        assert_eq!(
            AuthSchemeId::new("scheme_3"),
            config_bag
                .http_auth_schemes()
                .scheme(AuthSchemeId::new("scheme_3"))
                .unwrap()
                .scheme_id()
        );
    }
}

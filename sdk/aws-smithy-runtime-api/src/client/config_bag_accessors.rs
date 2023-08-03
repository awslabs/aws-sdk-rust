/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::auth::AuthOptionResolverParams;
use crate::client::orchestrator::{
    DynResponseDeserializer, EndpointResolverParams, LoadedRequestBody, ResponseDeserializer,
    SharedRequestSerializer, NOT_NEEDED,
};
use aws_smithy_types::config_bag::{CloneableLayer, ConfigBag, FrozenLayer, Layer};

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

use internal::{Gettable, Settable};

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

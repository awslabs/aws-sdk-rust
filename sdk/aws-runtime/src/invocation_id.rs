/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::BeforeTransmitInterceptorContextMut;
use aws_smithy_runtime_api::client::interceptors::Interceptor;
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreReplace};
use http::{HeaderName, HeaderValue};
use std::fmt::Debug;
use uuid::Uuid;

use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
#[cfg(feature = "test-util")]
pub use test_util::{NoInvocationIdGenerator, PredefinedInvocationIdGenerator};

#[allow(clippy::declare_interior_mutable_const)] // we will never mutate this
const AMZ_SDK_INVOCATION_ID: HeaderName = HeaderName::from_static("amz-sdk-invocation-id");

/// A generator for returning new invocation IDs on demand.
pub trait InvocationIdGenerator: Debug + Send + Sync {
    /// Call this function to receive a new [`InvocationId`] or an error explaining why one couldn't
    /// be provided.
    fn generate(&self) -> Result<Option<InvocationId>, BoxError>;
}

/// Dynamic dispatch implementation of [`InvocationIdGenerator`]
#[derive(Debug)]
pub struct DynInvocationIdGenerator(Box<dyn InvocationIdGenerator>);

impl DynInvocationIdGenerator {
    /// Creates a new [`DynInvocationIdGenerator`].
    pub fn new(gen: impl InvocationIdGenerator + 'static) -> Self {
        Self(Box::new(gen))
    }
}

impl InvocationIdGenerator for DynInvocationIdGenerator {
    fn generate(&self) -> Result<Option<InvocationId>, BoxError> {
        self.0.generate()
    }
}

impl Storable for DynInvocationIdGenerator {
    type Storer = StoreReplace<Self>;
}

/// This interceptor generates a UUID and attaches it to all request attempts made as part of this operation.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct InvocationIdInterceptor {}

impl InvocationIdInterceptor {
    /// Creates a new `InvocationIdInterceptor`
    pub fn new() -> Self {
        Self::default()
    }
}

impl Interceptor for InvocationIdInterceptor {
    fn modify_before_retry_loop(
        &self,
        _ctx: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let id = cfg
            .load::<DynInvocationIdGenerator>()
            .map(|gen| gen.generate())
            .transpose()?
            .flatten();
        cfg.interceptor_state()
            .store_put::<InvocationId>(id.unwrap_or_default());

        Ok(())
    }

    fn modify_before_transmit(
        &self,
        ctx: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let headers = ctx.request_mut().headers_mut();
        let id = cfg
            .load::<InvocationId>()
            .ok_or("Expected an InvocationId in the ConfigBag but none was present")?;
        headers.append(AMZ_SDK_INVOCATION_ID, id.0.clone());
        Ok(())
    }
}

/// InvocationId provides a consistent ID across retries
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationId(HeaderValue);

impl InvocationId {
    /// Create a new, random, invocation ID.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Defaults to a random UUID.
impl Default for InvocationId {
    fn default() -> Self {
        let id = Uuid::new_v4();
        let id = id
            .to_string()
            .parse()
            .expect("UUIDs always produce a valid header value");
        Self(id)
    }
}

impl Storable for InvocationId {
    type Storer = StoreReplace<Self>;
}

#[cfg(feature = "test-util")]
mod test_util {
    use super::*;
    use std::sync::{Arc, Mutex};

    impl InvocationId {
        /// Create a new invocation ID from a `&'static str`.
        pub fn new_from_str(uuid: &'static str) -> Self {
            InvocationId(HeaderValue::from_static(uuid))
        }
    }

    /// A "generator" that returns [`InvocationId`]s from a predefined list.
    #[derive(Debug)]
    pub struct PredefinedInvocationIdGenerator {
        pre_generated_ids: Arc<Mutex<Vec<InvocationId>>>,
    }

    impl PredefinedInvocationIdGenerator {
        /// Given a `Vec<InvocationId>`, create a new [`PredefinedInvocationIdGenerator`].
        pub fn new(mut invocation_ids: Vec<InvocationId>) -> Self {
            // We're going to pop ids off of the end of the list, so we need to reverse the list or else
            // we'll be popping the ids in reverse order, confusing the poor test writer.
            invocation_ids.reverse();

            Self {
                pre_generated_ids: Arc::new(Mutex::new(invocation_ids)),
            }
        }
    }

    impl InvocationIdGenerator for PredefinedInvocationIdGenerator {
        fn generate(&self) -> Result<Option<InvocationId>, BoxError> {
            Ok(Some(
                self.pre_generated_ids
                    .lock()
                    .expect("this will never be under contention")
                    .pop()
                    .expect("testers will provide enough invocation IDs"),
            ))
        }
    }

    /// A "generator" that always returns `None`.
    #[derive(Debug, Default)]
    pub struct NoInvocationIdGenerator;

    impl NoInvocationIdGenerator {
        /// Create a new [`NoInvocationIdGenerator`].
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl InvocationIdGenerator for NoInvocationIdGenerator {
        fn generate(&self) -> Result<Option<InvocationId>, BoxError> {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::invocation_id::{InvocationId, InvocationIdInterceptor};
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_runtime_api::client::interceptors::context::{
        BeforeTransmitInterceptorContextMut, InterceptorContext,
    };
    use aws_smithy_runtime_api::client::interceptors::Interceptor;
    use aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder;
    use aws_smithy_types::config_bag::ConfigBag;
    use aws_smithy_types::type_erasure::TypeErasedBox;
    use http::HeaderValue;

    fn expect_header<'a>(
        context: &'a BeforeTransmitInterceptorContextMut<'_>,
        header_name: &str,
    ) -> &'a HeaderValue {
        context.request().headers().get(header_name).unwrap()
    }

    #[test]
    fn test_id_is_generated_and_set() {
        let rc = RuntimeComponentsBuilder::for_tests().build().unwrap();
        let mut ctx = InterceptorContext::new(TypeErasedBox::doesnt_matter());
        ctx.enter_serialization_phase();
        ctx.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());
        let _ = ctx.take_input();
        ctx.enter_before_transmit_phase();

        let mut cfg = ConfigBag::base();
        let interceptor = InvocationIdInterceptor::new();
        let mut ctx = Into::into(&mut ctx);
        interceptor
            .modify_before_retry_loop(&mut ctx, &rc, &mut cfg)
            .unwrap();
        interceptor
            .modify_before_transmit(&mut ctx, &rc, &mut cfg)
            .unwrap();

        let expected = cfg.load::<InvocationId>().expect("invocation ID was set");
        let header = expect_header(&ctx, "amz-sdk-invocation-id");
        assert_eq!(expected.0, header, "the invocation ID in the config bag must match the invocation ID in the request header");
        // UUID should include 32 chars and 4 dashes
        assert_eq!(header.len(), 36);
    }
}

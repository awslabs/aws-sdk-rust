/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::interceptors::error::BoxError;
use aws_smithy_runtime_api::client::interceptors::{Interceptor, InterceptorContext};
use aws_smithy_runtime_api::config_bag::ConfigBag;
use http::{HeaderName, HeaderValue};
use uuid::Uuid;

#[allow(clippy::declare_interior_mutable_const)] // we will never mutate this
const AMZ_SDK_INVOCATION_ID: HeaderName = HeaderName::from_static("amz-sdk-invocation-id");

/// This interceptor generates a UUID and attaches it to all request attempts made as part of this operation.
#[non_exhaustive]
#[derive(Debug)]
pub struct InvocationIdInterceptor {
    id: InvocationId,
}

impl InvocationIdInterceptor {
    /// Creates a new `InvocationIdInterceptor`
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for InvocationIdInterceptor {
    fn default() -> Self {
        Self {
            id: InvocationId::from_uuid(),
        }
    }
}

impl Interceptor for InvocationIdInterceptor {
    fn modify_before_retry_loop(
        &self,
        context: &mut InterceptorContext,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let headers = context.request_mut()?.headers_mut();
        let id = _cfg.get::<InvocationId>().unwrap_or(&self.id);
        headers.append(AMZ_SDK_INVOCATION_ID, id.0.clone());
        Ok(())
    }
}

/// InvocationId provides a consistent ID across retries
#[derive(Debug)]
pub struct InvocationId(HeaderValue);
impl InvocationId {
    /// A test invocation id to allow deterministic requests
    pub fn for_tests() -> Self {
        InvocationId(HeaderValue::from_static(
            "00000000-0000-4000-8000-000000000000",
        ))
    }

    fn from_uuid() -> Self {
        let id = Uuid::new_v4();
        let id = id
            .to_string()
            .parse()
            .expect("UUIDs always produce a valid header value");
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use crate::invocation_id::InvocationIdInterceptor;
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_runtime_api::client::interceptors::{Interceptor, InterceptorContext};
    use aws_smithy_runtime_api::config_bag::ConfigBag;
    use aws_smithy_runtime_api::type_erasure::TypedBox;
    use http::HeaderValue;

    fn expect_header<'a>(context: &'a InterceptorContext, header_name: &str) -> &'a HeaderValue {
        context
            .request()
            .unwrap()
            .headers()
            .get(header_name)
            .unwrap()
    }

    #[test]
    fn test_id_is_generated_and_set() {
        let mut context = InterceptorContext::new(TypedBox::new("doesntmatter").erase());
        context.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());

        let mut config = ConfigBag::base();
        let interceptor = InvocationIdInterceptor::new();
        interceptor
            .modify_before_retry_loop(&mut context, &mut config)
            .unwrap();

        let header = expect_header(&context, "amz-sdk-invocation-id");
        assert_eq!(&interceptor.id.0, header);
        // UUID should include 32 chars and 4 dashes
        assert_eq!(interceptor.id.0.len(), 36);
    }
}

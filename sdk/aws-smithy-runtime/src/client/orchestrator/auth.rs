/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::interceptors::InterceptorContext;
use aws_smithy_runtime_api::client::orchestrator::{BoxError, ConfigBagAccessors};
use aws_smithy_runtime_api::config_bag::ConfigBag;
use std::fmt;

pub(super) async fn orchestrate_auth(
    ctx: &mut InterceptorContext,
    cfg: &ConfigBag,
) -> Result<(), BoxError> {
    let params = cfg.auth_option_resolver_params();
    let auth_options = cfg.auth_option_resolver().resolve_auth_options(params)?;
    let identity_resolvers = cfg.identity_resolvers();

    tracing::trace!(
        auth_option_resolver_params = ?params,
        auth_options = ?auth_options,
        identity_resolvers = ?identity_resolvers,
        "orchestrating auth",
    );
    for &scheme_id in auth_options.as_ref() {
        if let Some(auth_scheme) = cfg.http_auth_schemes().scheme(scheme_id) {
            if let Some(identity_resolver) = auth_scheme.identity_resolver(identity_resolvers) {
                let request_signer = auth_scheme.request_signer();

                let identity = identity_resolver.resolve_identity(cfg).await?;
                let request = ctx.request_mut();
                request_signer.sign_request(request, &identity, cfg)?;
                return Ok(());
            }
        }
    }

    Err(NoMatchingAuthScheme.into())
}

#[derive(Debug)]
struct NoMatchingAuthScheme;

impl fmt::Display for NoMatchingAuthScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "no auth scheme matched auth options. This is a bug. Please file an issue."
        )
    }
}

impl std::error::Error for NoMatchingAuthScheme {}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_runtime_api::client::auth::option_resolver::StaticAuthOptionResolver;
    use aws_smithy_runtime_api::client::auth::{
        AuthOptionResolverParams, AuthSchemeId, HttpAuthScheme, HttpAuthSchemes, HttpRequestSigner,
    };
    use aws_smithy_runtime_api::client::identity::{Identity, IdentityResolver, IdentityResolvers};
    use aws_smithy_runtime_api::client::interceptors::InterceptorContext;
    use aws_smithy_runtime_api::client::orchestrator::{Future, HttpRequest};
    use aws_smithy_runtime_api::type_erasure::TypedBox;

    #[tokio::test]
    async fn basic_case() {
        #[derive(Debug)]
        struct TestIdentityResolver;
        impl IdentityResolver for TestIdentityResolver {
            fn resolve_identity(&self, _config_bag: &ConfigBag) -> Future<Identity> {
                Future::ready(Ok(Identity::new("doesntmatter", None)))
            }
        }

        #[derive(Debug)]
        struct TestSigner;

        impl HttpRequestSigner for TestSigner {
            fn sign_request(
                &self,
                request: &mut HttpRequest,
                _identity: &Identity,
                _config_bag: &ConfigBag,
            ) -> Result<(), BoxError> {
                request
                    .headers_mut()
                    .insert(http::header::AUTHORIZATION, "success!".parse().unwrap());
                Ok(())
            }
        }

        const TEST_SCHEME_ID: AuthSchemeId = AuthSchemeId::new("test-scheme");

        #[derive(Debug)]
        struct TestAuthScheme {
            signer: TestSigner,
        }
        impl HttpAuthScheme for TestAuthScheme {
            fn scheme_id(&self) -> AuthSchemeId {
                TEST_SCHEME_ID
            }

            fn identity_resolver<'a>(
                &self,
                identity_resolvers: &'a IdentityResolvers,
            ) -> Option<&'a dyn IdentityResolver> {
                identity_resolvers.identity_resolver(self.scheme_id())
            }

            fn request_signer(&self) -> &dyn HttpRequestSigner {
                &self.signer
            }
        }

        let mut ctx = InterceptorContext::new(TypedBox::new("doesnt-matter").erase());
        ctx.enter_serialization_phase();
        ctx.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());
        let _ = ctx.take_input();
        ctx.enter_before_transmit_phase();

        let mut cfg = ConfigBag::base();
        cfg.set_auth_option_resolver_params(AuthOptionResolverParams::new("doesntmatter"));
        cfg.set_auth_option_resolver(StaticAuthOptionResolver::new(vec![TEST_SCHEME_ID]));
        cfg.set_identity_resolvers(
            IdentityResolvers::builder()
                .identity_resolver(TEST_SCHEME_ID, TestIdentityResolver)
                .build(),
        );
        cfg.set_http_auth_schemes(
            HttpAuthSchemes::builder()
                .auth_scheme(TEST_SCHEME_ID, TestAuthScheme { signer: TestSigner })
                .build(),
        );

        orchestrate_auth(&mut ctx, &cfg).await.expect("success");

        assert_eq!(
            "success!",
            ctx.request().headers().get("Authorization").unwrap()
        );
    }

    #[cfg(feature = "http-auth")]
    #[tokio::test]
    async fn select_best_scheme_for_available_identity_resolvers() {
        use crate::client::auth::http::{BasicAuthScheme, BearerAuthScheme};
        use aws_smithy_runtime_api::client::auth::http::{
            HTTP_BASIC_AUTH_SCHEME_ID, HTTP_BEARER_AUTH_SCHEME_ID,
        };
        use aws_smithy_runtime_api::client::identity::http::{Login, Token};

        let mut ctx = InterceptorContext::new(TypedBox::new("doesnt-matter").erase());
        ctx.enter_serialization_phase();
        ctx.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());
        let _ = ctx.take_input();
        ctx.enter_before_transmit_phase();

        let mut cfg = ConfigBag::base();
        cfg.set_auth_option_resolver_params(AuthOptionResolverParams::new("doesntmatter"));
        cfg.set_auth_option_resolver(StaticAuthOptionResolver::new(vec![
            HTTP_BASIC_AUTH_SCHEME_ID,
            HTTP_BEARER_AUTH_SCHEME_ID,
        ]));
        cfg.set_http_auth_schemes(
            HttpAuthSchemes::builder()
                .auth_scheme(HTTP_BASIC_AUTH_SCHEME_ID, BasicAuthScheme::new())
                .auth_scheme(HTTP_BEARER_AUTH_SCHEME_ID, BearerAuthScheme::new())
                .build(),
        );

        // First, test the presence of a basic auth login and absence of a bearer token
        cfg.set_identity_resolvers(
            IdentityResolvers::builder()
                .identity_resolver(HTTP_BASIC_AUTH_SCHEME_ID, Login::new("a", "b", None))
                .build(),
        );

        orchestrate_auth(&mut ctx, &cfg).await.expect("success");
        assert_eq!(
            // "YTpi" == "a:b" in base64
            "Basic YTpi",
            ctx.request().headers().get("Authorization").unwrap()
        );

        // Next, test the presence of a bearer token and absence of basic auth
        cfg.set_identity_resolvers(
            IdentityResolvers::builder()
                .identity_resolver(HTTP_BEARER_AUTH_SCHEME_ID, Token::new("t", None))
                .build(),
        );

        let mut ctx = InterceptorContext::new(TypedBox::new("doesnt-matter").erase());
        ctx.enter_serialization_phase();
        ctx.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());
        let _ = ctx.take_input();
        ctx.enter_before_transmit_phase();
        orchestrate_auth(&mut ctx, &cfg).await.expect("success");
        assert_eq!(
            "Bearer t",
            ctx.request().headers().get("Authorization").unwrap()
        );
    }
}

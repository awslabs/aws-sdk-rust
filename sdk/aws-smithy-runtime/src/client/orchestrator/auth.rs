/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::auth::{AuthSchemeEndpointConfig, AuthSchemeId};
use aws_smithy_runtime_api::client::interceptors::InterceptorContext;
use aws_smithy_runtime_api::client::orchestrator::{BoxError, ConfigBagAccessors};
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::endpoint::Endpoint;
use aws_smithy_types::Document;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
enum AuthOrchestrationError {
    NoMatchingAuthScheme,
    BadAuthSchemeEndpointConfig(Cow<'static, str>),
    AuthSchemeEndpointConfigMismatch(String),
}

impl AuthOrchestrationError {
    fn auth_scheme_endpoint_config_mismatch<'a>(
        auth_schemes: impl Iterator<Item = &'a Document>,
    ) -> Self {
        Self::AuthSchemeEndpointConfigMismatch(
            auth_schemes
                .flat_map(|s| match s {
                    Document::Object(map) => match map.get("name") {
                        Some(Document::String(name)) => Some(name.as_str()),
                        _ => None,
                    },
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

impl fmt::Display for AuthOrchestrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoMatchingAuthScheme => f.write_str(
                "no auth scheme matched auth options. This is a bug. Please file an issue.",
            ),
            Self::BadAuthSchemeEndpointConfig(message) => f.write_str(message),
            Self::AuthSchemeEndpointConfigMismatch(supported_schemes) => {
                write!(f,
                    "selected auth scheme / endpoint config mismatch. Couldn't find `sigv4` endpoint config for this endpoint. \
                    The authentication schemes supported by this endpoint are: {:?}",
                    supported_schemes
                )
            }
        }
    }
}

impl StdError for AuthOrchestrationError {}

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
                let endpoint = cfg
                    .get::<Endpoint>()
                    .expect("endpoint added to config bag by endpoint orchestrator");
                let auth_scheme_endpoint_config =
                    extract_endpoint_auth_scheme_config(endpoint, scheme_id)?;

                let identity = identity_resolver.resolve_identity(cfg).await?;
                let request = ctx.request_mut().expect("set during serialization");
                request_signer.sign_request(
                    request,
                    &identity,
                    auth_scheme_endpoint_config,
                    cfg,
                )?;
                return Ok(());
            }
        }
    }

    Err(AuthOrchestrationError::NoMatchingAuthScheme.into())
}

fn extract_endpoint_auth_scheme_config(
    endpoint: &Endpoint,
    scheme_id: AuthSchemeId,
) -> Result<AuthSchemeEndpointConfig<'_>, AuthOrchestrationError> {
    let auth_schemes = match endpoint.properties().get("authSchemes") {
        Some(Document::Array(schemes)) => schemes,
        // no auth schemes:
        None => return Ok(AuthSchemeEndpointConfig::new(None)),
        _other => {
            return Err(AuthOrchestrationError::BadAuthSchemeEndpointConfig(
                "expected an array for `authSchemes` in endpoint config".into(),
            ))
        }
    };
    let auth_scheme_config = auth_schemes
        .iter()
        .find(|doc| {
            let config_scheme_id = doc
                .as_object()
                .and_then(|object| object.get("name"))
                .and_then(Document::as_string);
            config_scheme_id == Some(scheme_id.as_str())
        })
        .ok_or_else(|| {
            AuthOrchestrationError::auth_scheme_endpoint_config_mismatch(auth_schemes.iter())
        })?;
    Ok(AuthSchemeEndpointConfig::new(Some(auth_scheme_config)))
}

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
    use aws_smithy_types::type_erasure::TypedBox;
    use std::collections::HashMap;

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
                _auth_scheme_endpoint_config: AuthSchemeEndpointConfig<'_>,
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
        cfg.put(Endpoint::builder().url("dontcare").build());

        orchestrate_auth(&mut ctx, &cfg).await.expect("success");

        assert_eq!(
            "success!",
            ctx.request()
                .expect("request is set")
                .headers()
                .get("Authorization")
                .unwrap()
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
        cfg.put(Endpoint::builder().url("dontcare").build());

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
            ctx.request()
                .expect("request is set")
                .headers()
                .get("Authorization")
                .unwrap()
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
            ctx.request()
                .expect("request is set")
                .headers()
                .get("Authorization")
                .unwrap()
        );
    }

    #[test]
    fn extract_endpoint_auth_scheme_config_no_config() {
        let endpoint = Endpoint::builder()
            .url("dontcare")
            .property("something-unrelated", Document::Null)
            .build();
        let config = extract_endpoint_auth_scheme_config(&endpoint, "test-scheme-id".into())
            .expect("success");
        assert!(config.config().is_none());
    }

    #[test]
    fn extract_endpoint_auth_scheme_config_wrong_type() {
        let endpoint = Endpoint::builder()
            .url("dontcare")
            .property("authSchemes", Document::String("bad".into()))
            .build();
        extract_endpoint_auth_scheme_config(&endpoint, "test-scheme-id".into())
            .expect_err("should fail because authSchemes is the wrong type");
    }

    #[test]
    fn extract_endpoint_auth_scheme_config_no_matching_scheme() {
        let endpoint = Endpoint::builder()
            .url("dontcare")
            .property(
                "authSchemes",
                vec![
                    Document::Object({
                        let mut out = HashMap::new();
                        out.insert("name".to_string(), "wrong-scheme-id".to_string().into());
                        out
                    }),
                    Document::Object({
                        let mut out = HashMap::new();
                        out.insert(
                            "name".to_string(),
                            "another-wrong-scheme-id".to_string().into(),
                        );
                        out
                    }),
                ],
            )
            .build();
        extract_endpoint_auth_scheme_config(&endpoint, "test-scheme-id".into())
            .expect_err("should fail because authSchemes doesn't include the desired scheme");
    }

    #[test]
    fn extract_endpoint_auth_scheme_config_successfully() {
        let endpoint = Endpoint::builder()
            .url("dontcare")
            .property(
                "authSchemes",
                vec![
                    Document::Object({
                        let mut out = HashMap::new();
                        out.insert("name".to_string(), "wrong-scheme-id".to_string().into());
                        out
                    }),
                    Document::Object({
                        let mut out = HashMap::new();
                        out.insert("name".to_string(), "test-scheme-id".to_string().into());
                        out.insert(
                            "magicString".to_string(),
                            "magic string value".to_string().into(),
                        );
                        out
                    }),
                ],
            )
            .build();
        let config = extract_endpoint_auth_scheme_config(&endpoint, "test-scheme-id".into())
            .expect("should find test-scheme-id");
        assert_eq!(
            "magic string value",
            config
                .config()
                .expect("config is set")
                .as_object()
                .expect("it's an object")
                .get("magicString")
                .expect("magicString is set")
                .as_string()
                .expect("gimme the string, dammit!")
        );
    }
}

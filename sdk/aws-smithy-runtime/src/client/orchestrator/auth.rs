/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::auth::no_auth::NO_AUTH_SCHEME_ID;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::auth::{
    AuthScheme, AuthSchemeEndpointConfig, AuthSchemeId, AuthSchemeOptionResolver,
    AuthSchemeOptionResolverParams,
};
use aws_smithy_runtime_api::client::identity::IdentityResolver;
use aws_smithy_runtime_api::client::interceptors::context::InterceptorContext;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::endpoint::Endpoint;
use aws_smithy_types::Document;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt;
use tracing::trace;

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
                "no auth scheme matched auth scheme options. This is a bug. Please file an issue.",
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
    runtime_components: &RuntimeComponents,
    cfg: &ConfigBag,
) -> Result<(), BoxError> {
    let params = cfg
        .load::<AuthSchemeOptionResolverParams>()
        .expect("auth scheme option resolver params must be set");
    let option_resolver = runtime_components.auth_scheme_option_resolver();
    let options = option_resolver.resolve_auth_scheme_options(params)?;

    trace!(
        auth_scheme_option_resolver_params = ?params,
        auth_scheme_options = ?options,
        "orchestrating auth",
    );

    for &scheme_id in options.as_ref() {
        if let Some(auth_scheme) = runtime_components.auth_scheme(scheme_id) {
            if let Some(identity_resolver) = auth_scheme.identity_resolver(runtime_components) {
                let signer = auth_scheme.signer();
                trace!(
                    auth_scheme = ?auth_scheme,
                    identity_resolver = ?identity_resolver,
                    signer = ?signer,
                    "resolved auth scheme, identity resolver, and signing implementation"
                );

                let endpoint = cfg
                    .load::<Endpoint>()
                    .expect("endpoint added to config bag by endpoint orchestrator");
                let auth_scheme_endpoint_config =
                    extract_endpoint_auth_scheme_config(endpoint, scheme_id)?;
                trace!(auth_scheme_endpoint_config = ?auth_scheme_endpoint_config, "extracted auth scheme endpoint config");

                let identity = identity_resolver.resolve_identity(cfg).await?;
                trace!(identity = ?identity, "resolved identity");

                trace!("signing request");
                let request = ctx.request_mut().expect("set during serialization");
                signer.sign_http_request(
                    request,
                    &identity,
                    auth_scheme_endpoint_config,
                    runtime_components,
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
    // TODO(P96049742): Endpoint config doesn't currently have a concept of optional auth or "no auth", so
    // we are short-circuiting lookup of endpoint auth scheme config if that is the selected scheme.
    if scheme_id == NO_AUTH_SCHEME_ID {
        return Ok(AuthSchemeEndpointConfig::empty());
    }
    let auth_schemes = match endpoint.properties().get("authSchemes") {
        Some(Document::Array(schemes)) => schemes,
        // no auth schemes:
        None => return Ok(AuthSchemeEndpointConfig::empty()),
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
    Ok(AuthSchemeEndpointConfig::from(Some(auth_scheme_config)))
}

#[cfg(all(test, feature = "test-util"))]
mod tests {
    use super::*;
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_runtime_api::client::auth::static_resolver::StaticAuthSchemeOptionResolver;
    use aws_smithy_runtime_api::client::auth::{
        AuthScheme, AuthSchemeId, AuthSchemeOptionResolverParams, SharedAuthScheme,
        SharedAuthSchemeOptionResolver, Signer,
    };
    use aws_smithy_runtime_api::client::identity::{
        Identity, IdentityResolver, SharedIdentityResolver,
    };
    use aws_smithy_runtime_api::client::interceptors::context::{Input, InterceptorContext};
    use aws_smithy_runtime_api::client::orchestrator::{Future, HttpRequest};
    use aws_smithy_runtime_api::client::runtime_components::{
        GetIdentityResolver, RuntimeComponentsBuilder,
    };
    use aws_smithy_types::config_bag::Layer;
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

        impl Signer for TestSigner {
            fn sign_http_request(
                &self,
                request: &mut HttpRequest,
                _identity: &Identity,
                _auth_scheme_endpoint_config: AuthSchemeEndpointConfig<'_>,
                _runtime_components: &RuntimeComponents,
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
        impl AuthScheme for TestAuthScheme {
            fn scheme_id(&self) -> AuthSchemeId {
                TEST_SCHEME_ID
            }

            fn identity_resolver(
                &self,
                identity_resolvers: &dyn GetIdentityResolver,
            ) -> Option<SharedIdentityResolver> {
                identity_resolvers.identity_resolver(self.scheme_id())
            }

            fn signer(&self) -> &dyn Signer {
                &self.signer
            }
        }

        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.enter_serialization_phase();
        ctx.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());
        let _ = ctx.take_input();
        ctx.enter_before_transmit_phase();

        let runtime_components = RuntimeComponentsBuilder::for_tests()
            .with_auth_scheme(SharedAuthScheme::new(TestAuthScheme { signer: TestSigner }))
            .with_auth_scheme_option_resolver(Some(SharedAuthSchemeOptionResolver::new(
                StaticAuthSchemeOptionResolver::new(vec![TEST_SCHEME_ID]),
            )))
            .with_identity_resolver(
                TEST_SCHEME_ID,
                SharedIdentityResolver::new(TestIdentityResolver),
            )
            .build()
            .unwrap();

        let mut layer: Layer = Layer::new("test");
        layer.store_put(AuthSchemeOptionResolverParams::new("doesntmatter"));
        layer.store_put(Endpoint::builder().url("dontcare").build());
        let cfg = ConfigBag::of_layers(vec![layer]);

        orchestrate_auth(&mut ctx, &runtime_components, &cfg)
            .await
            .expect("success");

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

        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.enter_serialization_phase();
        ctx.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());
        let _ = ctx.take_input();
        ctx.enter_before_transmit_phase();

        fn config_with_identity(
            scheme_id: AuthSchemeId,
            identity: impl IdentityResolver + 'static,
        ) -> (RuntimeComponents, ConfigBag) {
            let runtime_components = RuntimeComponentsBuilder::for_tests()
                .with_auth_scheme(SharedAuthScheme::new(BasicAuthScheme::new()))
                .with_auth_scheme(SharedAuthScheme::new(BearerAuthScheme::new()))
                .with_auth_scheme_option_resolver(Some(SharedAuthSchemeOptionResolver::new(
                    StaticAuthSchemeOptionResolver::new(vec![
                        HTTP_BASIC_AUTH_SCHEME_ID,
                        HTTP_BEARER_AUTH_SCHEME_ID,
                    ]),
                )))
                .with_identity_resolver(scheme_id, SharedIdentityResolver::new(identity))
                .build()
                .unwrap();

            let mut layer = Layer::new("test");
            layer.store_put(Endpoint::builder().url("dontcare").build());
            layer.store_put(AuthSchemeOptionResolverParams::new("doesntmatter"));

            (runtime_components, ConfigBag::of_layers(vec![layer]))
        }

        // First, test the presence of a basic auth login and absence of a bearer token
        let (runtime_components, cfg) =
            config_with_identity(HTTP_BASIC_AUTH_SCHEME_ID, Login::new("a", "b", None));
        orchestrate_auth(&mut ctx, &runtime_components, &cfg)
            .await
            .expect("success");
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
        let (runtime_components, cfg) =
            config_with_identity(HTTP_BEARER_AUTH_SCHEME_ID, Token::new("t", None));
        let mut ctx = InterceptorContext::new(Input::erase("doesnt-matter"));
        ctx.enter_serialization_phase();
        ctx.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());
        let _ = ctx.take_input();
        ctx.enter_before_transmit_phase();
        orchestrate_auth(&mut ctx, &runtime_components, &cfg)
            .await
            .expect("success");
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
        assert!(config.as_document().is_none());
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
                .as_document()
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

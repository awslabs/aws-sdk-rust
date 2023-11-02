/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! IMDS Token Middleware
//! Requests to IMDS are two part:
//! 1. A PUT request to the token API is made
//! 2. A GET request is made to the requested API. The Token is added as a header.
//!
//! This module implements a middleware that will:
//! - Load a token via the token API
//! - Cache the token according to the TTL
//! - Retry token loading when it fails
//! - Attach the token to the request in the `x-aws-ec2-metadata-token` header

use crate::imds::client::error::{ImdsError, TokenError, TokenErrorKind};
use aws_credential_types::cache::ExpiringCache;
use aws_smithy_async::time::SharedTimeSource;
use aws_smithy_http::body::SdkBody;
use aws_smithy_runtime::client::orchestrator::operation::Operation;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::auth::static_resolver::StaticAuthSchemeOptionResolver;
use aws_smithy_runtime_api::client::auth::{
    AuthScheme, AuthSchemeEndpointConfig, AuthSchemeId, Signer,
};
use aws_smithy_runtime_api::client::identity::{
    Identity, IdentityFuture, IdentityResolver, SharedIdentityResolver,
};
use aws_smithy_runtime_api::client::orchestrator::{HttpRequest, HttpResponse, OrchestratorError};
use aws_smithy_runtime_api::client::runtime_components::{
    GetIdentityResolver, RuntimeComponents, RuntimeComponentsBuilder,
};
use aws_smithy_runtime_api::client::runtime_plugin::{RuntimePlugin, SharedRuntimePlugin};
use aws_smithy_types::config_bag::ConfigBag;
use http::{HeaderValue, Uri};
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Token Refresh Buffer
///
/// Tokens are cached to remove the need to reload the token between subsequent requests. To ensure
/// that a request never fails with a 401 (expired token), a buffer window exists during which the token
/// may not be expired, but will still be refreshed.
const TOKEN_REFRESH_BUFFER: Duration = Duration::from_secs(120);

const X_AWS_EC2_METADATA_TOKEN_TTL_SECONDS: &str = "x-aws-ec2-metadata-token-ttl-seconds";
const X_AWS_EC2_METADATA_TOKEN: &str = "x-aws-ec2-metadata-token";
const IMDS_TOKEN_AUTH_SCHEME: AuthSchemeId = AuthSchemeId::new(X_AWS_EC2_METADATA_TOKEN);

/// IMDS Token
#[derive(Clone)]
struct Token {
    value: HeaderValue,
    expiry: SystemTime,
}
impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("value", &"** redacted **")
            .field("expiry", &self.expiry)
            .finish()
    }
}

/// Token Runtime Plugin
///
/// This runtime plugin wires up the necessary components to load/cache a token
/// when required and handle caching/expiry. This token will get attached to the
/// request to IMDS on the `x-aws-ec2-metadata-token` header.
#[derive(Debug)]
pub(super) struct TokenRuntimePlugin {
    components: RuntimeComponentsBuilder,
}

impl TokenRuntimePlugin {
    pub(super) fn new(
        common_plugin: SharedRuntimePlugin,
        time_source: SharedTimeSource,
        token_ttl: Duration,
    ) -> Self {
        Self {
            components: RuntimeComponentsBuilder::new("TokenRuntimePlugin")
                .with_auth_scheme(TokenAuthScheme::new())
                .with_auth_scheme_option_resolver(Some(StaticAuthSchemeOptionResolver::new(vec![
                    IMDS_TOKEN_AUTH_SCHEME,
                ])))
                .with_identity_resolver(
                    IMDS_TOKEN_AUTH_SCHEME,
                    TokenResolver::new(common_plugin, time_source, token_ttl),
                ),
        }
    }
}

impl RuntimePlugin for TokenRuntimePlugin {
    fn runtime_components(
        &self,
        _current_components: &RuntimeComponentsBuilder,
    ) -> Cow<'_, RuntimeComponentsBuilder> {
        Cow::Borrowed(&self.components)
    }
}

#[derive(Debug)]
struct TokenResolverInner {
    cache: ExpiringCache<Token, ImdsError>,
    refresh: Operation<(), Token, TokenError>,
    time_source: SharedTimeSource,
}

#[derive(Clone, Debug)]
struct TokenResolver {
    inner: Arc<TokenResolverInner>,
}

impl TokenResolver {
    fn new(
        common_plugin: SharedRuntimePlugin,
        time_source: SharedTimeSource,
        token_ttl: Duration,
    ) -> Self {
        Self {
            inner: Arc::new(TokenResolverInner {
                cache: ExpiringCache::new(TOKEN_REFRESH_BUFFER),
                refresh: Operation::builder()
                    .service_name("imds")
                    .operation_name("get-token")
                    .runtime_plugin(common_plugin)
                    .no_auth()
                    .with_connection_poisoning()
                    .serializer(move |_| {
                        Ok(http::Request::builder()
                            .method("PUT")
                            .uri(Uri::from_static("/latest/api/token"))
                            .header(X_AWS_EC2_METADATA_TOKEN_TTL_SECONDS, token_ttl.as_secs())
                            .body(SdkBody::empty())
                            .expect("valid HTTP request"))
                    })
                    .deserializer({
                        let time_source = time_source.clone();
                        move |response| {
                            let now = time_source.now();
                            parse_token_response(response, now)
                                .map_err(OrchestratorError::operation)
                        }
                    })
                    .build(),
                time_source,
            }),
        }
    }

    async fn get_token(&self) -> Result<(Token, SystemTime), ImdsError> {
        self.inner
            .refresh
            .invoke(())
            .await
            .map(|token| {
                let expiry = token.expiry;
                (token, expiry)
            })
            .map_err(ImdsError::failed_to_load_token)
    }
}

fn parse_token_response(response: &HttpResponse, now: SystemTime) -> Result<Token, TokenError> {
    match response.status().as_u16() {
        400 => return Err(TokenErrorKind::InvalidParameters.into()),
        403 => return Err(TokenErrorKind::Forbidden.into()),
        _ => {}
    }
    let mut value =
        HeaderValue::from_bytes(response.body().bytes().expect("non-streaming response"))
            .map_err(|_| TokenErrorKind::InvalidToken)?;
    value.set_sensitive(true);

    let ttl: u64 = response
        .headers()
        .get(X_AWS_EC2_METADATA_TOKEN_TTL_SECONDS)
        .ok_or(TokenErrorKind::NoTtl)?
        .to_str()
        .map_err(|_| TokenErrorKind::InvalidTtl)?
        .parse()
        .map_err(|_parse_error| TokenErrorKind::InvalidTtl)?;
    Ok(Token {
        value,
        expiry: now + Duration::from_secs(ttl),
    })
}

impl IdentityResolver for TokenResolver {
    fn resolve_identity(&self, _config_bag: &ConfigBag) -> IdentityFuture {
        let this = self.clone();
        IdentityFuture::new(async move {
            let preloaded_token = this
                .inner
                .cache
                .yield_or_clear_if_expired(this.inner.time_source.now())
                .await;
            let token = match preloaded_token {
                Some(token) => Ok(token),
                None => {
                    this.inner
                        .cache
                        .get_or_load(|| {
                            let this = this.clone();
                            async move { this.get_token().await }
                        })
                        .await
                }
            }?;

            let expiry = token.expiry;
            Ok(Identity::new(token, Some(expiry)))
        })
    }
}

#[derive(Debug)]
struct TokenAuthScheme {
    signer: TokenSigner,
}

impl TokenAuthScheme {
    fn new() -> Self {
        Self {
            signer: TokenSigner,
        }
    }
}

impl AuthScheme for TokenAuthScheme {
    fn scheme_id(&self) -> AuthSchemeId {
        IMDS_TOKEN_AUTH_SCHEME
    }

    fn identity_resolver(
        &self,
        identity_resolvers: &dyn GetIdentityResolver,
    ) -> Option<SharedIdentityResolver> {
        identity_resolvers.identity_resolver(IMDS_TOKEN_AUTH_SCHEME)
    }

    fn signer(&self) -> &dyn Signer {
        &self.signer
    }
}

#[derive(Debug)]
struct TokenSigner;

impl Signer for TokenSigner {
    fn sign_http_request(
        &self,
        request: &mut HttpRequest,
        identity: &Identity,
        _auth_scheme_endpoint_config: AuthSchemeEndpointConfig<'_>,
        _runtime_components: &RuntimeComponents,
        _config_bag: &ConfigBag,
    ) -> Result<(), BoxError> {
        let token = identity.data::<Token>().expect("correct type");
        request
            .headers_mut()
            .append(X_AWS_EC2_METADATA_TOKEN, token.value.clone());
        Ok(())
    }
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! APIs for request authentication.

use crate::box_error::BoxError;
use crate::client::identity::{Identity, SharedIdentityResolver};
use crate::client::orchestrator::HttpRequest;
use crate::client::runtime_components::sealed::ValidateConfig;
use crate::client::runtime_components::{GetIdentityResolver, RuntimeComponents};
use crate::impl_shared_conversions;
use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer, Storable, StoreReplace};
use aws_smithy_types::type_erasure::TypeErasedBox;
use aws_smithy_types::Document;
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

/// Auth schemes for the HTTP `Authorization` header.
#[cfg(feature = "http-auth")]
pub mod http;

/// Static auth scheme option resolver.
pub mod static_resolver;

/// The output type from the [`ResolveAuthSchemeOptions::resolve_auth_scheme_options_v2`]
///
/// The resolver returns a list of these, in the order the auth scheme resolver wishes to use them.
#[derive(Clone, Debug)]
pub struct AuthSchemeOption {
    scheme_id: AuthSchemeId,
    properties: Option<FrozenLayer>,
}

impl AuthSchemeOption {
    /// Builder struct for [`AuthSchemeOption`]
    pub fn builder() -> AuthSchemeOptionBuilder {
        AuthSchemeOptionBuilder::default()
    }

    /// Returns [`AuthSchemeId`], the ID of the scheme
    pub fn scheme_id(&self) -> &AuthSchemeId {
        &self.scheme_id
    }

    /// Returns optional properties for identity resolution or signing
    ///
    /// This config layer is applied to the [`ConfigBag`] to ensure the information is
    /// available during both the identity resolution and signature generation processes.
    pub fn properties(&self) -> Option<FrozenLayer> {
        self.properties.clone()
    }
}

/// Builder struct for [`AuthSchemeOption`]
#[derive(Debug, Default)]
pub struct AuthSchemeOptionBuilder {
    scheme_id: Option<AuthSchemeId>,
    properties: Option<FrozenLayer>,
}

impl AuthSchemeOptionBuilder {
    /// Sets [`AuthSchemeId`] for the builder
    pub fn scheme_id(mut self, auth_scheme_id: AuthSchemeId) -> Self {
        self.set_scheme_id(Some(auth_scheme_id));
        self
    }

    /// Sets [`AuthSchemeId`] for the builder
    pub fn set_scheme_id(&mut self, auth_scheme_id: Option<AuthSchemeId>) {
        self.scheme_id = auth_scheme_id;
    }

    /// Sets the properties for the builder
    pub fn properties(mut self, properties: FrozenLayer) -> Self {
        self.set_properties(Some(properties));
        self
    }

    /// Sets the properties for the builder
    pub fn set_properties(&mut self, properties: Option<FrozenLayer>) {
        self.properties = properties;
    }

    /// Builds an [`AuthSchemeOption`], otherwise returns an [`AuthSchemeOptionBuilderError`] in the case of error
    pub fn build(self) -> Result<AuthSchemeOption, AuthSchemeOptionBuilderError> {
        let scheme_id = self
            .scheme_id
            .ok_or(ErrorKind::MissingRequiredField("auth_scheme_id"))?;
        Ok(AuthSchemeOption {
            scheme_id,
            properties: self.properties,
        })
    }
}

#[derive(Debug)]
enum ErrorKind {
    MissingRequiredField(&'static str),
}

impl From<ErrorKind> for AuthSchemeOptionBuilderError {
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

/// The error type returned when failing to build [`AuthSchemeOption`] from the builder
#[derive(Debug)]
pub struct AuthSchemeOptionBuilderError {
    kind: ErrorKind,
}

impl fmt::Display for AuthSchemeOptionBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::MissingRequiredField(name) => {
                write!(f, "`{name}` is required")
            }
        }
    }
}

impl std::error::Error for AuthSchemeOptionBuilderError {}

/// New type around an auth scheme ID.
///
/// Each auth scheme must have a unique string identifier associated with it,
/// which is used to refer to auth schemes by the auth scheme option resolver, and
/// also used to select an identity resolver to use.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct AuthSchemeId {
    scheme_id: Cow<'static, str>,
}

// See: https://doc.rust-lang.org/std/convert/trait.AsRef.html#reflexivity
impl AsRef<AuthSchemeId> for AuthSchemeId {
    fn as_ref(&self) -> &AuthSchemeId {
        self
    }
}

impl AuthSchemeId {
    /// Creates a new auth scheme ID.
    pub const fn new(scheme_id: &'static str) -> Self {
        Self {
            scheme_id: Cow::Borrowed(scheme_id),
        }
    }

    /// Returns the string equivalent of this auth scheme ID.
    #[deprecated(
        note = "This function is no longer functional. Use `inner` instead",
        since = "1.8.0"
    )]
    pub const fn as_str(&self) -> &'static str {
        match self.scheme_id {
            Cow::Borrowed(val) => val,
            Cow::Owned(_) => {
                // cannot obtain `&'static str` from `String` unless we use `Box::leak`
                ""
            }
        }
    }

    /// Returns the string equivalent of this auth scheme ID.
    pub fn inner(&self) -> &str {
        &self.scheme_id
    }
}

impl From<&'static str> for AuthSchemeId {
    fn from(scheme_id: &'static str) -> Self {
        Self::new(scheme_id)
    }
}

impl From<Cow<'static, str>> for AuthSchemeId {
    fn from(scheme_id: Cow<'static, str>) -> Self {
        Self { scheme_id }
    }
}

/// Parameters needed to resolve auth scheme options.
///
/// Most generated clients will use the [`StaticAuthSchemeOptionResolver`](static_resolver::StaticAuthSchemeOptionResolver),
/// which doesn't require any parameters for resolution (and has its own empty params struct).
///
/// However, more complex auth scheme resolvers may need modeled parameters in order to resolve
/// the auth scheme options. For those, this params struct holds a type erased box so that any
/// kind of parameters can be contained within, and type casted by the auth scheme option resolver
/// implementation.
#[derive(Debug)]
pub struct AuthSchemeOptionResolverParams(TypeErasedBox);

impl AuthSchemeOptionResolverParams {
    /// Creates a new [`AuthSchemeOptionResolverParams`].
    pub fn new<T: fmt::Debug + Send + Sync + 'static>(params: T) -> Self {
        Self(TypeErasedBox::new(params))
    }

    /// Returns the underlying parameters as the type `T` if they are that type.
    pub fn get<T: fmt::Debug + Send + Sync + 'static>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
}

impl Storable for AuthSchemeOptionResolverParams {
    type Storer = StoreReplace<Self>;
}

new_type_future! {
    #[doc = "Future for [`ResolveAuthSchemeOptions::resolve_auth_scheme_options_v2`]."]
    pub struct AuthSchemeOptionsFuture<'a, Vec<AuthSchemeOption>, BoxError>;
}

/// Resolver for auth scheme options.
///
/// The orchestrator needs to select an auth scheme to sign requests with, and potentially
/// from several different available auth schemes. Smithy models have a number of ways
/// to specify which operations can use which auth schemes under which conditions, as
/// documented in the [Smithy spec](https://smithy.io/2.0/spec/authentication-traits.html).
///
/// The orchestrator uses the auth scheme option resolver runtime component to resolve
/// an ordered list of options that are available to choose from for a given request.
/// This resolver can be a simple static list, such as with the
/// [`StaticAuthSchemeOptionResolver`](static_resolver::StaticAuthSchemeOptionResolver),
/// or it can be a complex code generated resolver that incorporates parameters from both
/// the model and the resolved endpoint.
pub trait ResolveAuthSchemeOptions: Send + Sync + fmt::Debug {
    #[deprecated(
        note = "This method is deprecated, use `resolve_auth_scheme_options_v2` instead.",
        since = "1.8.0"
    )]
    /// Returns a list of available auth scheme options to choose from.
    fn resolve_auth_scheme_options(
        &self,
        _params: &AuthSchemeOptionResolverParams,
    ) -> Result<Cow<'_, [AuthSchemeId]>, BoxError> {
        unimplemented!("This method is deprecated, use `resolve_auth_scheme_options_v2` instead.");
    }

    #[allow(deprecated)]
    /// Returns a list of available auth scheme options to choose from.
    fn resolve_auth_scheme_options_v2<'a>(
        &'a self,
        params: &'a AuthSchemeOptionResolverParams,
        _cfg: &'a ConfigBag,
        _runtime_components: &'a RuntimeComponents,
    ) -> AuthSchemeOptionsFuture<'a> {
        AuthSchemeOptionsFuture::ready({
            self.resolve_auth_scheme_options(params).map(|options| {
                options
                    .iter()
                    .cloned()
                    .map(|scheme_id| {
                        AuthSchemeOption::builder()
                            .scheme_id(scheme_id)
                            .build()
                            .expect("required fields set")
                    })
                    .collect::<Vec<_>>()
            })
        })
    }
}

/// A shared auth scheme option resolver.
#[derive(Clone, Debug)]
pub struct SharedAuthSchemeOptionResolver(Arc<dyn ResolveAuthSchemeOptions>);

impl SharedAuthSchemeOptionResolver {
    /// Creates a new [`SharedAuthSchemeOptionResolver`].
    pub fn new(auth_scheme_option_resolver: impl ResolveAuthSchemeOptions + 'static) -> Self {
        Self(Arc::new(auth_scheme_option_resolver))
    }
}

impl ResolveAuthSchemeOptions for SharedAuthSchemeOptionResolver {
    #[allow(deprecated)]
    fn resolve_auth_scheme_options(
        &self,
        params: &AuthSchemeOptionResolverParams,
    ) -> Result<Cow<'_, [AuthSchemeId]>, BoxError> {
        (*self.0).resolve_auth_scheme_options(params)
    }

    fn resolve_auth_scheme_options_v2<'a>(
        &'a self,
        params: &'a AuthSchemeOptionResolverParams,
        cfg: &'a ConfigBag,
        runtime_components: &'a RuntimeComponents,
    ) -> AuthSchemeOptionsFuture<'a> {
        (*self.0).resolve_auth_scheme_options_v2(params, cfg, runtime_components)
    }
}

impl_shared_conversions!(
    convert SharedAuthSchemeOptionResolver
    from ResolveAuthSchemeOptions
    using SharedAuthSchemeOptionResolver::new
);

/// An auth scheme.
///
/// Auth schemes have unique identifiers (the `scheme_id`),
/// and provide an identity resolver and a signer.
pub trait AuthScheme: Send + Sync + fmt::Debug {
    /// Returns the unique identifier associated with this auth scheme.
    ///
    /// This identifier is used to refer to this auth scheme from the
    /// [`ResolveAuthSchemeOptions`], and is also associated with
    /// identity resolvers in the config.
    fn scheme_id(&self) -> AuthSchemeId;

    /// Returns the identity resolver that can resolve an identity for this scheme, if one is available.
    ///
    /// The [`AuthScheme`] doesn't actually own an identity resolver. Rather, identity resolvers
    /// are configured as runtime components. The auth scheme merely chooses a compatible identity
    /// resolver from the runtime components via the [`GetIdentityResolver`] trait. The trait is
    /// given rather than the full set of runtime components to prevent complex resolution logic
    /// involving multiple components from taking place in this function, since that's not the
    /// intended use of this design.
    fn identity_resolver(
        &self,
        identity_resolvers: &dyn GetIdentityResolver,
    ) -> Option<SharedIdentityResolver>;

    /// Returns the signing implementation for this auth scheme.
    fn signer(&self) -> &dyn Sign;
}

/// Container for a shared auth scheme implementation.
#[derive(Clone, Debug)]
pub struct SharedAuthScheme(Arc<dyn AuthScheme>);

impl SharedAuthScheme {
    /// Creates a new [`SharedAuthScheme`] from the given auth scheme.
    pub fn new(auth_scheme: impl AuthScheme + 'static) -> Self {
        Self(Arc::new(auth_scheme))
    }
}

impl AuthScheme for SharedAuthScheme {
    fn scheme_id(&self) -> AuthSchemeId {
        self.0.scheme_id()
    }

    fn identity_resolver(
        &self,
        identity_resolvers: &dyn GetIdentityResolver,
    ) -> Option<SharedIdentityResolver> {
        self.0.identity_resolver(identity_resolvers)
    }

    fn signer(&self) -> &dyn Sign {
        self.0.signer()
    }
}

impl ValidateConfig for SharedAuthScheme {}

impl_shared_conversions!(convert SharedAuthScheme from AuthScheme using SharedAuthScheme::new);

/// Signing implementation for an auth scheme.
pub trait Sign: Send + Sync + fmt::Debug {
    /// Sign the given request with the given identity, components, and config.
    ///
    /// If the provided identity is incompatible with this signer, an error must be returned.
    fn sign_http_request(
        &self,
        request: &mut HttpRequest,
        identity: &Identity,
        auth_scheme_endpoint_config: AuthSchemeEndpointConfig<'_>,
        runtime_components: &RuntimeComponents,
        config_bag: &ConfigBag,
    ) -> Result<(), BoxError>;
}

/// Endpoint configuration for the selected auth scheme.
///
/// The configuration held by this struct originates from the endpoint rule set in the service model.
///
/// This struct gets added to the request state by the auth orchestrator.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct AuthSchemeEndpointConfig<'a>(Option<&'a Document>);

impl<'a> AuthSchemeEndpointConfig<'a> {
    /// Creates an empty [`AuthSchemeEndpointConfig`].
    pub fn empty() -> Self {
        Self(None)
    }

    /// Returns the endpoint configuration as a [`Document`].
    pub fn as_document(&self) -> Option<&'a Document> {
        self.0
    }
}

impl<'a> From<Option<&'a Document>> for AuthSchemeEndpointConfig<'a> {
    fn from(value: Option<&'a Document>) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a Document> for AuthSchemeEndpointConfig<'a> {
    fn from(value: &'a Document) -> Self {
        Self(Some(value))
    }
}

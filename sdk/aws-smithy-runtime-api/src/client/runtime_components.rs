/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Runtime components used to make a request and handle a response.
//!
//! Runtime components are trait implementations that are _always_ used by the orchestrator.
//! There are other trait implementations that can be configured for a client, but if they
//! aren't directly and always used by the orchestrator, then they are placed in the
//! [`ConfigBag`](aws_smithy_types::config_bag::ConfigBag) instead of in
//! [`RuntimeComponents`](RuntimeComponents).

use crate::client::auth::{
    AuthScheme, AuthSchemeId, SharedAuthScheme, SharedAuthSchemeOptionResolver,
};
use crate::client::connectors::SharedHttpConnector;
use crate::client::endpoint::SharedEndpointResolver;
use crate::client::identity::{ConfiguredIdentityResolver, SharedIdentityResolver};
use crate::client::interceptors::SharedInterceptor;
use crate::client::retries::{RetryClassifiers, SharedRetryStrategy};
use aws_smithy_async::rt::sleep::SharedAsyncSleep;
use aws_smithy_async::time::SharedTimeSource;
use std::fmt;

pub(crate) static EMPTY_RUNTIME_COMPONENTS_BUILDER: RuntimeComponentsBuilder =
    RuntimeComponentsBuilder::new("empty");

/// Internal to `declare_runtime_components!`.
///
/// Merges a field from one builder into another.
macro_rules! merge {
    (Option $other:ident . $name:ident => $self:ident) => {
        $self.$name = $other.$name.clone().or($self.$name.take());
    };
    (Vec $other:ident . $name:ident => $self:ident) => {
        if !$other.$name.is_empty() {
            $self.$name.extend($other.$name.iter().cloned());
        }
    };
}
/// Internal to `declare_runtime_components!`.
///
/// This is used when creating the builder's `build` method
/// to populate each individual field value. The `required`/`atLeastOneRequired`
/// validations are performed here.
macro_rules! builder_field_value {
    (Option $self:ident . $name:ident) => {
        $self.$name
    };
    (Option $self:ident . $name:ident required) => {
        $self.$name.ok_or(BuildError(concat!(
            "the `",
            stringify!($name),
            "` runtime component is required"
        )))?
    };
    (Vec $self:ident . $name:ident) => {
        $self.$name
    };
    (Vec $self:ident . $name:ident atLeastOneRequired) => {{
        if $self.$name.is_empty() {
            return Err(BuildError(concat!(
                "at least one `",
                stringify!($name),
                "` runtime component is required"
            )));
        }
        $self.$name
    }};
}
/// Internal to `declare_runtime_components!`.
///
/// Converts the field type from `Option<T>` or `Vec<T>` into `Option<Tracked<T>>` or `Vec<Tracked<T>>` respectively.
/// Also removes the `Option` wrapper for required fields in the non-builder struct.
macro_rules! runtime_component_field_type {
    (Option $inner_type:ident) => {
        Option<Tracked<$inner_type>>
    };
    (Option $inner_type:ident required) => {
        Tracked<$inner_type>
    };
    (Vec $inner_type:ident) => {
        Vec<Tracked<$inner_type>>
    };
    (Vec $inner_type:ident atLeastOneRequired) => {
        Vec<Tracked<$inner_type>>
    };
}
/// Internal to `declare_runtime_components!`.
///
/// Converts an `$outer_type` into an empty instantiation for that type.
/// This is needed since `Default::default()` can't be used in a `const` function,
/// and `RuntimeComponentsBuilder::new()` is `const`.
macro_rules! empty_builder_value {
    (Option) => {
        None
    };
    (Vec) => {
        Vec::new()
    };
}

/// Macro to define the structs for both `RuntimeComponents` and `RuntimeComponentsBuilder`.
///
/// This is a macro in order to keep the fields consistent between the two, and to automatically
/// update the `merge_from` and `build` methods when new components are added.
///
/// It also facilitates unit testing since the overall mechanism can be unit tested with different
/// fields that are easy to check in tests (testing with real components makes it hard
/// to tell that the correct component was selected when merging builders).
///
/// # Example usage
///
/// The two identifiers after "fields for" become the names of the struct and builder respectively.
/// Following that, all the fields are specified. Fields MUST be wrapped in `Option` or `Vec`.
/// To make a field required in the non-builder struct, add `#[required]` for `Option` fields, or
/// `#[atLeastOneRequired]` for `Vec` fields.
///
/// ```no_compile
/// declare_runtime_components! {
///     fields for TestRc and TestRcBuilder {
///         some_optional_string: Option<String>,
///
///         some_optional_vec: Vec<String>,
///
///         #[required]
///         some_required_string: Option<String>,
///
///         #[atLeastOneRequired]
///         some_required_vec: Vec<String>,
///     }
/// }
/// ```
macro_rules! declare_runtime_components {
    (fields for $rc_name:ident and $builder_name:ident {
        $($(#[$option:ident])? $field_name:ident : $outer_type:ident<$inner_type:ident> ,)+
    }) => {
        /// Components that can only be set in runtime plugins that the orchestrator uses directly to call an operation.
        #[derive(Clone, Debug)]
        pub struct $rc_name {
            $($field_name: runtime_component_field_type!($outer_type $inner_type $($option)?),)+
        }

        /// Builder for [`RuntimeComponents`].
        #[derive(Clone, Debug)]
        pub struct $builder_name {
            builder_name: &'static str,
            $($field_name: $outer_type<Tracked<$inner_type>>,)+
        }
        impl $builder_name {
            /// Creates a new builder.
            ///
            /// Since multiple builders are merged together to make the final [`RuntimeComponents`],
            /// all components added by this builder are associated with the given `name` so that
            /// the origin of a component can be easily found when debugging.
            pub const fn new(name: &'static str) -> Self {
                Self {
                    builder_name: name,
                    $($field_name: empty_builder_value!($outer_type),)+
                }
            }

            /// Merge in components from another builder.
            pub fn merge_from(mut self, other: &Self) -> Self {
                $(merge!($outer_type other.$field_name => self);)+
                self
            }

            /// Builds [`RuntimeComponents`] from this builder.
            pub fn build(self) -> Result<$rc_name, BuildError> {
                Ok($rc_name {
                    $($field_name: builder_field_value!($outer_type self.$field_name $($option)?),)+
                })
            }
        }
    };
}

declare_runtime_components! {
    fields for RuntimeComponents and RuntimeComponentsBuilder {
        #[required]
        auth_scheme_option_resolver: Option<SharedAuthSchemeOptionResolver>,

        // A connector is not required since a client could technically only be used for presigning
        http_connector: Option<SharedHttpConnector>,

        #[required]
        endpoint_resolver: Option<SharedEndpointResolver>,

        #[atLeastOneRequired]
        auth_schemes: Vec<SharedAuthScheme>,

        #[atLeastOneRequired]
        identity_resolvers: Vec<ConfiguredIdentityResolver>,

        interceptors: Vec<SharedInterceptor>,

        retry_classifiers: Option<RetryClassifiers>,

        #[required]
        retry_strategy: Option<SharedRetryStrategy>,

        time_source: Option<SharedTimeSource>,

        sleep_impl: Option<SharedAsyncSleep>,
    }
}

impl RuntimeComponents {
    /// Returns a builder for runtime components.
    pub fn builder(name: &'static str) -> RuntimeComponentsBuilder {
        RuntimeComponentsBuilder::new(name)
    }

    /// Returns the auth scheme option resolver.
    pub fn auth_scheme_option_resolver(&self) -> SharedAuthSchemeOptionResolver {
        self.auth_scheme_option_resolver.value.clone()
    }

    /// Returns the connector.
    pub fn http_connector(&self) -> Option<SharedHttpConnector> {
        self.http_connector.as_ref().map(|s| s.value.clone())
    }

    /// Returns the endpoint resolver.
    pub fn endpoint_resolver(&self) -> SharedEndpointResolver {
        self.endpoint_resolver.value.clone()
    }

    /// Returns the requested auth scheme if it is set.
    pub fn auth_scheme(&self, scheme_id: AuthSchemeId) -> Option<SharedAuthScheme> {
        self.auth_schemes
            .iter()
            .find(|s| s.value.scheme_id() == scheme_id)
            .map(|s| s.value.clone())
    }

    /// Returns an iterator over the interceptors.
    pub fn interceptors(&self) -> impl Iterator<Item = SharedInterceptor> + '_ {
        self.interceptors.iter().map(|s| s.value.clone())
    }

    /// Returns the retry classifiers.
    pub fn retry_classifiers(&self) -> Option<&RetryClassifiers> {
        self.retry_classifiers.as_ref().map(|s| &s.value)
    }

    /// Returns the retry strategy.
    pub fn retry_strategy(&self) -> SharedRetryStrategy {
        self.retry_strategy.value.clone()
    }

    /// Returns the async sleep implementation.
    pub fn sleep_impl(&self) -> Option<SharedAsyncSleep> {
        self.sleep_impl.as_ref().map(|s| s.value.clone())
    }

    /// Returns the time source.
    pub fn time_source(&self) -> Option<SharedTimeSource> {
        self.time_source.as_ref().map(|s| s.value.clone())
    }
}

impl RuntimeComponentsBuilder {
    /// Returns the auth scheme option resolver.
    pub fn auth_scheme_option_resolver(&self) -> Option<SharedAuthSchemeOptionResolver> {
        self.auth_scheme_option_resolver
            .as_ref()
            .map(|s| s.value.clone())
    }

    /// Sets the auth scheme option resolver.
    pub fn set_auth_scheme_option_resolver(
        &mut self,
        auth_scheme_option_resolver: Option<SharedAuthSchemeOptionResolver>,
    ) -> &mut Self {
        self.auth_scheme_option_resolver =
            auth_scheme_option_resolver.map(|r| Tracked::new(self.builder_name, r));
        self
    }

    /// Sets the auth scheme option resolver.
    pub fn with_auth_scheme_option_resolver(
        mut self,
        auth_scheme_option_resolver: Option<SharedAuthSchemeOptionResolver>,
    ) -> Self {
        self.set_auth_scheme_option_resolver(auth_scheme_option_resolver);
        self
    }

    /// Returns the HTTP connector.
    pub fn http_connector(&self) -> Option<SharedHttpConnector> {
        self.http_connector.as_ref().map(|s| s.value.clone())
    }

    /// Sets the HTTP connector.
    pub fn set_http_connector(&mut self, connector: Option<SharedHttpConnector>) -> &mut Self {
        self.http_connector = connector.map(|c| Tracked::new(self.builder_name, c));
        self
    }

    /// Sets the HTTP connector.
    pub fn with_http_connector(mut self, connector: Option<SharedHttpConnector>) -> Self {
        self.set_http_connector(connector);
        self
    }

    /// Returns the endpoint resolver.
    pub fn endpoint_resolver(&self) -> Option<SharedEndpointResolver> {
        self.endpoint_resolver.as_ref().map(|s| s.value.clone())
    }

    /// Sets the endpoint resolver.
    pub fn set_endpoint_resolver(
        &mut self,
        endpoint_resolver: Option<SharedEndpointResolver>,
    ) -> &mut Self {
        self.endpoint_resolver = endpoint_resolver.map(|s| Tracked::new(self.builder_name, s));
        self
    }

    /// Sets the endpoint resolver.
    pub fn with_endpoint_resolver(
        mut self,
        endpoint_resolver: Option<SharedEndpointResolver>,
    ) -> Self {
        self.set_endpoint_resolver(endpoint_resolver);
        self
    }

    /// Returns the auth schemes.
    pub fn auth_schemes(&self) -> impl Iterator<Item = SharedAuthScheme> + '_ {
        self.auth_schemes.iter().map(|s| s.value.clone())
    }

    /// Adds an auth scheme.
    pub fn push_auth_scheme(&mut self, auth_scheme: SharedAuthScheme) -> &mut Self {
        self.auth_schemes
            .push(Tracked::new(self.builder_name, auth_scheme));
        self
    }

    /// Adds an auth scheme.
    pub fn with_auth_scheme(mut self, auth_scheme: SharedAuthScheme) -> Self {
        self.push_auth_scheme(auth_scheme);
        self
    }

    /// Adds an identity resolver.
    pub fn push_identity_resolver(
        &mut self,
        scheme_id: AuthSchemeId,
        identity_resolver: SharedIdentityResolver,
    ) -> &mut Self {
        self.identity_resolvers.push(Tracked::new(
            self.builder_name,
            ConfiguredIdentityResolver::new(scheme_id, identity_resolver),
        ));
        self
    }

    /// Adds an identity resolver.
    pub fn with_identity_resolver(
        mut self,
        scheme_id: AuthSchemeId,
        identity_resolver: SharedIdentityResolver,
    ) -> Self {
        self.push_identity_resolver(scheme_id, identity_resolver);
        self
    }

    /// Returns the interceptors.
    pub fn interceptors(&self) -> impl Iterator<Item = SharedInterceptor> + '_ {
        self.interceptors.iter().map(|s| s.value.clone())
    }

    /// Adds all the given interceptors.
    pub fn extend_interceptors(
        &mut self,
        interceptors: impl Iterator<Item = SharedInterceptor>,
    ) -> &mut Self {
        self.interceptors
            .extend(interceptors.map(|s| Tracked::new(self.builder_name, s)));
        self
    }

    /// Adds an interceptor.
    pub fn push_interceptor(&mut self, interceptor: SharedInterceptor) -> &mut Self {
        self.interceptors
            .push(Tracked::new(self.builder_name, interceptor));
        self
    }

    /// Adds an interceptor.
    pub fn with_interceptor(mut self, interceptor: SharedInterceptor) -> Self {
        self.push_interceptor(interceptor);
        self
    }

    /// Directly sets the interceptors and clears out any that were previously pushed.
    pub fn set_interceptors(
        &mut self,
        interceptors: impl Iterator<Item = SharedInterceptor>,
    ) -> &mut Self {
        self.interceptors.clear();
        self.interceptors
            .extend(interceptors.map(|s| Tracked::new(self.builder_name, s)));
        self
    }

    /// Directly sets the interceptors and clears out any that were previously pushed.
    pub fn with_interceptors(
        mut self,
        interceptors: impl Iterator<Item = SharedInterceptor>,
    ) -> Self {
        self.set_interceptors(interceptors);
        self
    }

    /// Returns the retry classifiers.
    pub fn retry_classifiers(&self) -> Option<&RetryClassifiers> {
        self.retry_classifiers.as_ref().map(|s| &s.value)
    }

    /// Sets the retry classifiers.
    pub fn set_retry_classifiers(
        &mut self,
        retry_classifiers: Option<RetryClassifiers>,
    ) -> &mut Self {
        self.retry_classifiers = retry_classifiers.map(|s| Tracked::new(self.builder_name, s));
        self
    }

    /// Sets the retry classifiers.
    pub fn with_retry_classifiers(mut self, retry_classifiers: Option<RetryClassifiers>) -> Self {
        self.retry_classifiers = retry_classifiers.map(|s| Tracked::new(self.builder_name, s));
        self
    }

    /// Returns the retry strategy.
    pub fn retry_strategy(&self) -> Option<SharedRetryStrategy> {
        self.retry_strategy.as_ref().map(|s| s.value.clone())
    }

    /// Sets the retry strategy.
    pub fn set_retry_strategy(&mut self, retry_strategy: Option<SharedRetryStrategy>) -> &mut Self {
        self.retry_strategy = retry_strategy.map(|s| Tracked::new(self.builder_name, s));
        self
    }

    /// Sets the retry strategy.
    pub fn with_retry_strategy(mut self, retry_strategy: Option<SharedRetryStrategy>) -> Self {
        self.retry_strategy = retry_strategy.map(|s| Tracked::new(self.builder_name, s));
        self
    }

    /// Returns the async sleep implementation.
    pub fn sleep_impl(&self) -> Option<SharedAsyncSleep> {
        self.sleep_impl.as_ref().map(|s| s.value.clone())
    }

    /// Sets the async sleep implementation.
    pub fn set_sleep_impl(&mut self, sleep_impl: Option<SharedAsyncSleep>) -> &mut Self {
        self.sleep_impl = sleep_impl.map(|s| Tracked::new(self.builder_name, s));
        self
    }

    /// Sets the async sleep implementation.
    pub fn with_sleep_impl(mut self, sleep_impl: Option<SharedAsyncSleep>) -> Self {
        self.sleep_impl = sleep_impl.map(|s| Tracked::new(self.builder_name, s));
        self
    }

    /// Returns the time source.
    pub fn time_source(&self) -> Option<SharedTimeSource> {
        self.time_source.as_ref().map(|s| s.value.clone())
    }

    /// Sets the time source.
    pub fn set_time_source(&mut self, time_source: Option<SharedTimeSource>) -> &mut Self {
        self.time_source = time_source.map(|s| Tracked::new(self.builder_name, s));
        self
    }

    /// Sets the time source.
    pub fn with_time_source(mut self, time_source: Option<SharedTimeSource>) -> Self {
        self.time_source = time_source.map(|s| Tracked::new(self.builder_name, s));
        self
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
struct Tracked<T> {
    _origin: &'static str,
    value: T,
}

impl<T> Tracked<T> {
    fn new(origin: &'static str, value: T) -> Self {
        Self {
            _origin: origin,
            value,
        }
    }
}

impl RuntimeComponentsBuilder {
    /// Creates a runtime components builder with all the required components filled in with fake (panicking) implementations.
    #[cfg(feature = "test-util")]
    pub fn for_tests() -> Self {
        use crate::client::auth::AuthSchemeOptionResolver;
        use crate::client::connectors::HttpConnector;
        use crate::client::endpoint::{EndpointResolver, EndpointResolverParams};
        use crate::client::identity::Identity;
        use crate::client::identity::IdentityResolver;
        use crate::client::orchestrator::Future;
        use crate::client::retries::RetryStrategy;
        use aws_smithy_async::rt::sleep::AsyncSleep;
        use aws_smithy_async::time::TimeSource;
        use aws_smithy_types::config_bag::ConfigBag;
        use aws_smithy_types::endpoint::Endpoint;

        #[derive(Debug)]
        struct FakeAuthSchemeOptionResolver;
        impl AuthSchemeOptionResolver for FakeAuthSchemeOptionResolver {
            fn resolve_auth_scheme_options(
                &self,
                _: &crate::client::auth::AuthSchemeOptionResolverParams,
            ) -> Result<std::borrow::Cow<'_, [AuthSchemeId]>, crate::box_error::BoxError>
            {
                unreachable!("fake auth scheme option resolver must be overridden for this test")
            }
        }

        #[derive(Debug)]
        struct FakeConnector;
        impl HttpConnector for FakeConnector {
            fn call(
                &self,
                _: crate::client::orchestrator::HttpRequest,
            ) -> crate::client::orchestrator::BoxFuture<crate::client::orchestrator::HttpResponse>
            {
                unreachable!("fake connector must be overridden for this test")
            }
        }

        #[derive(Debug)]
        struct FakeEndpointResolver;
        impl EndpointResolver for FakeEndpointResolver {
            fn resolve_endpoint(&self, _: &EndpointResolverParams) -> Future<Endpoint> {
                unreachable!("fake endpoint resolver must be overridden for this test")
            }
        }

        #[derive(Debug)]
        struct FakeAuthScheme;
        impl AuthScheme for FakeAuthScheme {
            fn scheme_id(&self) -> AuthSchemeId {
                AuthSchemeId::new("fake")
            }

            fn identity_resolver(
                &self,
                _: &dyn GetIdentityResolver,
            ) -> Option<SharedIdentityResolver> {
                None
            }

            fn signer(&self) -> &dyn crate::client::auth::Signer {
                unreachable!("fake http auth scheme must be overridden for this test")
            }
        }

        #[derive(Debug)]
        struct FakeIdentityResolver;
        impl IdentityResolver for FakeIdentityResolver {
            fn resolve_identity(&self, _: &ConfigBag) -> Future<Identity> {
                unreachable!("fake identity resolver must be overridden for this test")
            }
        }

        #[derive(Debug)]
        struct FakeRetryStrategy;
        impl RetryStrategy for FakeRetryStrategy {
            fn should_attempt_initial_request(
                &self,
                _: &RuntimeComponents,
                _: &ConfigBag,
            ) -> Result<crate::client::retries::ShouldAttempt, crate::box_error::BoxError>
            {
                unreachable!("fake retry strategy must be overridden for this test")
            }

            fn should_attempt_retry(
                &self,
                _: &crate::client::interceptors::context::InterceptorContext,
                _: &RuntimeComponents,
                _: &ConfigBag,
            ) -> Result<crate::client::retries::ShouldAttempt, crate::box_error::BoxError>
            {
                unreachable!("fake retry strategy must be overridden for this test")
            }
        }

        #[derive(Debug)]
        struct FakeTimeSource;
        impl TimeSource for FakeTimeSource {
            fn now(&self) -> std::time::SystemTime {
                unreachable!("fake time source must be overridden for this test")
            }
        }

        #[derive(Debug)]
        struct FakeSleep;
        impl AsyncSleep for FakeSleep {
            fn sleep(&self, _: std::time::Duration) -> aws_smithy_async::rt::sleep::Sleep {
                unreachable!("fake sleep must be overridden for this test")
            }
        }

        Self::new("aws_smithy_runtime_api::client::runtime_components::RuntimeComponentBuilder::for_tests")
            .with_auth_scheme(SharedAuthScheme::new(FakeAuthScheme))
            .with_auth_scheme_option_resolver(Some(SharedAuthSchemeOptionResolver::new(FakeAuthSchemeOptionResolver)))
            .with_endpoint_resolver(Some(SharedEndpointResolver::new(FakeEndpointResolver)))
            .with_http_connector(Some(SharedHttpConnector::new(FakeConnector)))
            .with_identity_resolver(AuthSchemeId::new("fake"), SharedIdentityResolver::new(FakeIdentityResolver))
            .with_retry_classifiers(Some(RetryClassifiers::new()))
            .with_retry_strategy(Some(SharedRetryStrategy::new(FakeRetryStrategy)))
            .with_sleep_impl(Some(SharedAsyncSleep::new(FakeSleep)))
            .with_time_source(Some(SharedTimeSource::new(FakeTimeSource)))
    }
}

/// An error that occurs when building runtime components.
#[derive(Debug)]
pub struct BuildError(&'static str);

impl std::error::Error for BuildError {}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A trait for retrieving a shared identity resolver.
///
/// This trait exists so that [`AuthScheme::identity_resolver`](crate::client::auth::AuthScheme::identity_resolver)
/// can have access to configured identity resolvers without having access to all the runtime components.
pub trait GetIdentityResolver: Send + Sync {
    /// Returns the requested identity resolver if it is set.
    fn identity_resolver(&self, scheme_id: AuthSchemeId) -> Option<SharedIdentityResolver>;
}

impl GetIdentityResolver for RuntimeComponents {
    fn identity_resolver(&self, scheme_id: AuthSchemeId) -> Option<SharedIdentityResolver> {
        self.identity_resolvers
            .iter()
            .find(|s| s.value.scheme_id() == scheme_id)
            .map(|s| s.value.identity_resolver())
    }
}

#[cfg(all(test, feature = "test-util"))]
mod tests {
    use super::*;

    #[test]
    #[allow(unreachable_pub)]
    #[allow(dead_code)]
    fn the_builders_should_merge() {
        declare_runtime_components! {
            fields for TestRc and TestRcBuilder {
                #[required]
                some_required_string: Option<String>,

                some_optional_string: Option<String>,

                #[atLeastOneRequired]
                some_required_vec: Vec<String>,

                some_optional_vec: Vec<String>,
            }
        }

        let builder1 = TestRcBuilder {
            builder_name: "builder1",
            some_required_string: Some(Tracked::new("builder1", "override_me".into())),
            some_optional_string: Some(Tracked::new("builder1", "override_me optional".into())),
            some_required_vec: vec![Tracked::new("builder1", "first".into())],
            some_optional_vec: vec![Tracked::new("builder1", "first optional".into())],
        };
        let builder2 = TestRcBuilder {
            builder_name: "builder2",
            some_required_string: Some(Tracked::new("builder2", "override_me_too".into())),
            some_optional_string: Some(Tracked::new("builder2", "override_me_too optional".into())),
            some_required_vec: vec![Tracked::new("builder2", "second".into())],
            some_optional_vec: vec![Tracked::new("builder2", "second optional".into())],
        };
        let builder3 = TestRcBuilder {
            builder_name: "builder3",
            some_required_string: Some(Tracked::new("builder3", "correct".into())),
            some_optional_string: Some(Tracked::new("builder3", "correct optional".into())),
            some_required_vec: vec![Tracked::new("builder3", "third".into())],
            some_optional_vec: vec![Tracked::new("builder3", "third optional".into())],
        };
        let rc = TestRcBuilder::new("root")
            .merge_from(&builder1)
            .merge_from(&builder2)
            .merge_from(&builder3)
            .build()
            .expect("success");
        assert_eq!(
            Tracked::new("builder3", "correct".to_string()),
            rc.some_required_string
        );
        assert_eq!(
            Some(Tracked::new("builder3", "correct optional".to_string())),
            rc.some_optional_string
        );
        assert_eq!(
            vec![
                Tracked::new("builder1", "first".to_string()),
                Tracked::new("builder2", "second".into()),
                Tracked::new("builder3", "third".into())
            ],
            rc.some_required_vec
        );
        assert_eq!(
            vec![
                Tracked::new("builder1", "first optional".to_string()),
                Tracked::new("builder2", "second optional".into()),
                Tracked::new("builder3", "third optional".into())
            ],
            rc.some_optional_vec
        );
    }

    #[test]
    #[allow(unreachable_pub)]
    #[allow(dead_code)]
    #[should_panic(expected = "the `_some_string` runtime component is required")]
    fn require_field_singular() {
        declare_runtime_components! {
            fields for TestRc and TestRcBuilder {
                #[required]
                _some_string: Option<String>,
            }
        }

        let rc = TestRcBuilder::new("test").build().unwrap();

        // Ensure the correct types were used
        let _: Tracked<String> = rc._some_string;
    }

    #[test]
    #[allow(unreachable_pub)]
    #[allow(dead_code)]
    #[should_panic(expected = "at least one `_some_vec` runtime component is required")]
    fn require_field_plural() {
        declare_runtime_components! {
            fields for TestRc and TestRcBuilder {
                #[atLeastOneRequired]
                _some_vec: Vec<String>,
            }
        }

        let rc = TestRcBuilder::new("test").build().unwrap();

        // Ensure the correct types were used
        let _: Vec<Tracked<String>> = rc._some_vec;
    }

    #[test]
    #[allow(unreachable_pub)]
    #[allow(dead_code)]
    fn optional_fields_dont_panic() {
        declare_runtime_components! {
            fields for TestRc and TestRcBuilder {
                _some_optional_string: Option<String>,
                _some_optional_vec: Vec<String>,
            }
        }

        let rc = TestRcBuilder::new("test").build().unwrap();

        // Ensure the correct types were used
        let _: Option<Tracked<String>> = rc._some_optional_string;
        let _: Vec<Tracked<String>> = rc._some_optional_vec;
    }

    #[test]
    fn building_test_builder_should_not_panic() {
        let _ = RuntimeComponentsBuilder::for_tests().build(); // should not panic
    }
}

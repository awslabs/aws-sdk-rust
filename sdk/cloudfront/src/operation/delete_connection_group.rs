// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
/// Orchestration and serialization glue logic for `DeleteConnectionGroup`.
#[derive(::std::clone::Clone, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct DeleteConnectionGroup;
impl DeleteConnectionGroup {
    /// Creates a new `DeleteConnectionGroup`
    pub fn new() -> Self {
        Self
    }
    pub(crate) async fn orchestrate(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::delete_connection_group::DeleteConnectionGroupInput,
    ) -> ::std::result::Result<
        crate::operation::delete_connection_group::DeleteConnectionGroupOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::delete_connection_group::DeleteConnectionGroupError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let map_err = |err: ::aws_smithy_runtime_api::client::result::SdkError<
            ::aws_smithy_runtime_api::client::interceptors::context::Error,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >| {
            err.map_service_error(|err| {
                err.downcast::<crate::operation::delete_connection_group::DeleteConnectionGroupError>()
                    .expect("correct error type")
            })
        };
        let context = Self::orchestrate_with_stop_point(runtime_plugins, input, ::aws_smithy_runtime::client::orchestrator::StopPoint::None)
            .await
            .map_err(map_err)?;
        let output = context.finalize().map_err(map_err)?;
        ::std::result::Result::Ok(
            output
                .downcast::<crate::operation::delete_connection_group::DeleteConnectionGroupOutput>()
                .expect("correct output type"),
        )
    }

    pub(crate) async fn orchestrate_with_stop_point(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::delete_connection_group::DeleteConnectionGroupInput,
        stop_point: ::aws_smithy_runtime::client::orchestrator::StopPoint,
    ) -> ::std::result::Result<
        ::aws_smithy_runtime_api::client::interceptors::context::InterceptorContext,
        ::aws_smithy_runtime_api::client::result::SdkError<
            ::aws_smithy_runtime_api::client::interceptors::context::Error,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let input = ::aws_smithy_runtime_api::client::interceptors::context::Input::erase(input);
        use ::tracing::Instrument;
        ::aws_smithy_runtime::client::orchestrator::invoke_with_stop_point("CloudFront", "DeleteConnectionGroup", input, runtime_plugins, stop_point)
            // Create a parent span for the entire operation. Includes a random, internal-only,
            // seven-digit ID for the operation orchestration so that it can be correlated in the logs.
            .instrument(::tracing::debug_span!(
                "CloudFront.DeleteConnectionGroup",
                "rpc.service" = "CloudFront",
                "rpc.method" = "DeleteConnectionGroup",
                "sdk_invocation_id" = ::fastrand::u32(1_000_000..10_000_000),
                "rpc.system" = "aws-api",
            ))
            .await
    }

    pub(crate) fn operation_runtime_plugins(
        client_runtime_plugins: ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        client_config: &crate::config::Config,
        config_override: ::std::option::Option<crate::config::Builder>,
    ) -> ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins {
        let mut runtime_plugins = client_runtime_plugins.with_operation_plugin(Self::new());

        if let ::std::option::Option::Some(config_override) = config_override {
            for plugin in config_override.runtime_plugins.iter().cloned() {
                runtime_plugins = runtime_plugins.with_operation_plugin(plugin);
            }
            runtime_plugins = runtime_plugins.with_operation_plugin(crate::config::ConfigOverrideRuntimePlugin::new(
                config_override,
                client_config.config.clone(),
                &client_config.runtime_components,
            ));
        }
        runtime_plugins
    }
}
impl ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin for DeleteConnectionGroup {
    fn config(&self) -> ::std::option::Option<::aws_smithy_types::config_bag::FrozenLayer> {
        let mut cfg = ::aws_smithy_types::config_bag::Layer::new("DeleteConnectionGroup");

        cfg.store_put(::aws_smithy_runtime_api::client::ser_de::SharedRequestSerializer::new(
            DeleteConnectionGroupRequestSerializer,
        ));
        cfg.store_put(::aws_smithy_runtime_api::client::ser_de::SharedResponseDeserializer::new(
            DeleteConnectionGroupResponseDeserializer,
        ));

        cfg.store_put(::aws_smithy_runtime_api::client::auth::AuthSchemeOptionResolverParams::new(
            crate::config::auth::Params::builder()
                .operation_name("DeleteConnectionGroup")
                .build()
                .expect("required fields set"),
        ));

        cfg.store_put(::aws_smithy_runtime_api::client::orchestrator::Metadata::new(
            "DeleteConnectionGroup",
            "CloudFront",
        ));
        let mut signing_options = ::aws_runtime::auth::SigningOptions::default();
        signing_options.double_uri_encode = true;
        signing_options.content_sha256_header = false;
        signing_options.normalize_uri_path = true;
        signing_options.payload_override = None;

        cfg.store_put(::aws_runtime::auth::SigV4OperationSigningConfig {
            signing_options,
            ..::std::default::Default::default()
        });

        ::std::option::Option::Some(cfg.freeze())
    }

    fn runtime_components(
        &self,
        _: &::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder,
    ) -> ::std::borrow::Cow<'_, ::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder> {
        #[allow(unused_mut)]
        let mut rcb = ::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder::new("DeleteConnectionGroup")
            .with_interceptor(::aws_smithy_runtime::client::stalled_stream_protection::StalledStreamProtectionInterceptor::default())
            .with_interceptor(DeleteConnectionGroupEndpointParamsInterceptor)
            .with_retry_classifier(::aws_smithy_runtime::client::retries::classifiers::TransientErrorClassifier::<
                crate::operation::delete_connection_group::DeleteConnectionGroupError,
            >::new())
            .with_retry_classifier(::aws_smithy_runtime::client::retries::classifiers::ModeledAsRetryableClassifier::<
                crate::operation::delete_connection_group::DeleteConnectionGroupError,
            >::new())
            .with_retry_classifier(::aws_runtime::retries::classifiers::AwsErrorCodeClassifier::<
                crate::operation::delete_connection_group::DeleteConnectionGroupError,
            >::new());

        ::std::borrow::Cow::Owned(rcb)
    }
}

#[derive(Debug)]
struct DeleteConnectionGroupResponseDeserializer;
impl ::aws_smithy_runtime_api::client::ser_de::DeserializeResponse for DeleteConnectionGroupResponseDeserializer {
    fn deserialize_nonstreaming(
        &self,
        response: &::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
    ) -> ::aws_smithy_runtime_api::client::interceptors::context::OutputOrError {
        let (success, status) = (response.status().is_success(), response.status().as_u16());
        let headers = response.headers();
        let body = response.body().bytes().expect("body loaded");
        #[allow(unused_mut)]
        let mut force_error = false;
        ::tracing::debug!(request_id = ?::aws_types::request_id::RequestId::request_id(response));
        let parse_result = if !success && status != 204 || force_error {
            crate::protocol_serde::shape_delete_connection_group::de_delete_connection_group_http_error(status, headers, body)
        } else {
            crate::protocol_serde::shape_delete_connection_group::de_delete_connection_group_http_response(status, headers, body)
        };
        crate::protocol_serde::type_erase_result(parse_result)
    }
}
#[derive(Debug)]
struct DeleteConnectionGroupRequestSerializer;
impl ::aws_smithy_runtime_api::client::ser_de::SerializeRequest for DeleteConnectionGroupRequestSerializer {
    #[allow(unused_mut, clippy::let_and_return, clippy::needless_borrow, clippy::useless_conversion)]
    fn serialize_input(
        &self,
        input: ::aws_smithy_runtime_api::client::interceptors::context::Input,
        _cfg: &mut ::aws_smithy_types::config_bag::ConfigBag,
    ) -> ::std::result::Result<::aws_smithy_runtime_api::client::orchestrator::HttpRequest, ::aws_smithy_runtime_api::box_error::BoxError> {
        let input = input
            .downcast::<crate::operation::delete_connection_group::DeleteConnectionGroupInput>()
            .expect("correct type");
        let _header_serialization_settings = _cfg
            .load::<crate::serialization_settings::HeaderSerializationSettings>()
            .cloned()
            .unwrap_or_default();
        let mut request_builder = {
            fn uri_base(
                _input: &crate::operation::delete_connection_group::DeleteConnectionGroupInput,
                output: &mut ::std::string::String,
            ) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::BuildError> {
                use ::std::fmt::Write as _;
                let input_1 = &_input.id;
                let input_1 = input_1
                    .as_ref()
                    .ok_or_else(|| ::aws_smithy_types::error::operation::BuildError::missing_field("id", "cannot be empty or unset"))?;
                let id = ::aws_smithy_http::label::fmt_string(input_1, ::aws_smithy_http::label::EncodingStrategy::Default);
                if id.is_empty() {
                    return ::std::result::Result::Err(::aws_smithy_types::error::operation::BuildError::missing_field(
                        "id",
                        "cannot be empty or unset",
                    ));
                }
                ::std::write!(output, "/2020-05-31/connection-group/{Id}", Id = id).expect("formatting should succeed");
                ::std::result::Result::Ok(())
            }
            #[allow(clippy::unnecessary_wraps)]
            fn update_http_builder(
                input: &crate::operation::delete_connection_group::DeleteConnectionGroupInput,
                builder: ::http::request::Builder,
            ) -> ::std::result::Result<::http::request::Builder, ::aws_smithy_types::error::operation::BuildError> {
                let mut uri = ::std::string::String::new();
                uri_base(input, &mut uri)?;
                let builder = crate::protocol_serde::shape_delete_connection_group::ser_delete_connection_group_headers(input, builder)?;
                ::std::result::Result::Ok(builder.method("DELETE").uri(uri))
            }
            let mut builder = update_http_builder(&input, ::http::request::Builder::new())?;
            builder
        };
        let body = ::aws_smithy_types::body::SdkBody::from("");

        ::std::result::Result::Ok(request_builder.body(body).expect("valid request").try_into().unwrap())
    }
}
#[derive(Debug)]
struct DeleteConnectionGroupEndpointParamsInterceptor;

impl ::aws_smithy_runtime_api::client::interceptors::Intercept for DeleteConnectionGroupEndpointParamsInterceptor {
    fn name(&self) -> &'static str {
        "DeleteConnectionGroupEndpointParamsInterceptor"
    }

    fn read_before_execution(
        &self,
        context: &::aws_smithy_runtime_api::client::interceptors::context::BeforeSerializationInterceptorContextRef<
            '_,
            ::aws_smithy_runtime_api::client::interceptors::context::Input,
            ::aws_smithy_runtime_api::client::interceptors::context::Output,
            ::aws_smithy_runtime_api::client::interceptors::context::Error,
        >,
        cfg: &mut ::aws_smithy_types::config_bag::ConfigBag,
    ) -> ::std::result::Result<(), ::aws_smithy_runtime_api::box_error::BoxError> {
        let _input = context
            .input()
            .downcast_ref::<DeleteConnectionGroupInput>()
            .ok_or("failed to downcast to DeleteConnectionGroupInput")?;

        let params = crate::config::endpoint::Params::builder()
            .set_use_dual_stack(cfg.load::<::aws_types::endpoint_config::UseDualStack>().map(|ty| ty.0))
            .set_use_fips(cfg.load::<::aws_types::endpoint_config::UseFips>().map(|ty| ty.0))
            .set_endpoint(cfg.load::<::aws_types::endpoint_config::EndpointUrl>().map(|ty| ty.0.clone()))
            .set_region(cfg.load::<::aws_types::region::Region>().map(|r| r.as_ref().to_owned()))
            .build()
            .map_err(|err| {
                ::aws_smithy_runtime_api::client::interceptors::error::ContextAttachedError::new("endpoint params could not be built", err)
            })?;
        cfg.interceptor_state()
            .store_put(::aws_smithy_runtime_api::client::endpoint::EndpointResolverParams::new(params));
        ::std::result::Result::Ok(())
    }
}

// The get_* functions below are generated from JMESPath expressions in the
// operationContextParams trait. They target the operation's input shape.

/// Error type for the `DeleteConnectionGroupError` operation.
#[non_exhaustive]
#[derive(::std::fmt::Debug)]
pub enum DeleteConnectionGroupError {
    /// <p>Access denied.</p>
    AccessDenied(crate::types::error::AccessDenied),
    /// <p>The entity cannot be deleted while it is in use.</p>
    CannotDeleteEntityWhileInUse(crate::types::error::CannotDeleteEntityWhileInUse),
    /// <p>The entity was not found.</p>
    EntityNotFound(crate::types::error::EntityNotFound),
    /// <p>The <code>If-Match</code> version is missing or not valid.</p>
    InvalidIfMatchVersion(crate::types::error::InvalidIfMatchVersion),
    /// <p>The precondition in one or more of the request fields evaluated to <code>false</code>.</p>
    PreconditionFailed(crate::types::error::PreconditionFailed),
    /// <p>The specified CloudFront resource hasn't been disabled yet.</p>
    ResourceNotDisabled(crate::types::error::ResourceNotDisabled),
    /// An unexpected error occurred (e.g., invalid JSON returned by the service or an unknown error code).
    #[deprecated(note = "Matching `Unhandled` directly is not forwards compatible. Instead, match using a \
    variable wildcard pattern and check `.code()`:
     \
    &nbsp;&nbsp;&nbsp;`err if err.code() == Some(\"SpecificExceptionCode\") => { /* handle the error */ }`
     \
    See [`ProvideErrorMetadata`](#impl-ProvideErrorMetadata-for-DeleteConnectionGroupError) for what information is available for the error.")]
    Unhandled(crate::error::sealed_unhandled::Unhandled),
}
impl DeleteConnectionGroupError {
    /// Creates the `DeleteConnectionGroupError::Unhandled` variant from any error type.
    pub fn unhandled(
        err: impl ::std::convert::Into<::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static>>,
    ) -> Self {
        Self::Unhandled(crate::error::sealed_unhandled::Unhandled {
            source: err.into(),
            meta: ::std::default::Default::default(),
        })
    }

    /// Creates the `DeleteConnectionGroupError::Unhandled` variant from an [`ErrorMetadata`](::aws_smithy_types::error::ErrorMetadata).
    pub fn generic(err: ::aws_smithy_types::error::ErrorMetadata) -> Self {
        Self::Unhandled(crate::error::sealed_unhandled::Unhandled {
            source: err.clone().into(),
            meta: err,
        })
    }
    ///
    /// Returns error metadata, which includes the error code, message,
    /// request ID, and potentially additional information.
    ///
    pub fn meta(&self) -> &::aws_smithy_types::error::ErrorMetadata {
        match self {
            Self::AccessDenied(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::CannotDeleteEntityWhileInUse(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::EntityNotFound(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::InvalidIfMatchVersion(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::PreconditionFailed(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::ResourceNotDisabled(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::Unhandled(e) => &e.meta,
        }
    }
    /// Returns `true` if the error kind is `DeleteConnectionGroupError::AccessDenied`.
    pub fn is_access_denied(&self) -> bool {
        matches!(self, Self::AccessDenied(_))
    }
    /// Returns `true` if the error kind is `DeleteConnectionGroupError::CannotDeleteEntityWhileInUse`.
    pub fn is_cannot_delete_entity_while_in_use(&self) -> bool {
        matches!(self, Self::CannotDeleteEntityWhileInUse(_))
    }
    /// Returns `true` if the error kind is `DeleteConnectionGroupError::EntityNotFound`.
    pub fn is_entity_not_found(&self) -> bool {
        matches!(self, Self::EntityNotFound(_))
    }
    /// Returns `true` if the error kind is `DeleteConnectionGroupError::InvalidIfMatchVersion`.
    pub fn is_invalid_if_match_version(&self) -> bool {
        matches!(self, Self::InvalidIfMatchVersion(_))
    }
    /// Returns `true` if the error kind is `DeleteConnectionGroupError::PreconditionFailed`.
    pub fn is_precondition_failed(&self) -> bool {
        matches!(self, Self::PreconditionFailed(_))
    }
    /// Returns `true` if the error kind is `DeleteConnectionGroupError::ResourceNotDisabled`.
    pub fn is_resource_not_disabled(&self) -> bool {
        matches!(self, Self::ResourceNotDisabled(_))
    }
}
impl ::std::error::Error for DeleteConnectionGroupError {
    fn source(&self) -> ::std::option::Option<&(dyn ::std::error::Error + 'static)> {
        match self {
            Self::AccessDenied(_inner) => ::std::option::Option::Some(_inner),
            Self::CannotDeleteEntityWhileInUse(_inner) => ::std::option::Option::Some(_inner),
            Self::EntityNotFound(_inner) => ::std::option::Option::Some(_inner),
            Self::InvalidIfMatchVersion(_inner) => ::std::option::Option::Some(_inner),
            Self::PreconditionFailed(_inner) => ::std::option::Option::Some(_inner),
            Self::ResourceNotDisabled(_inner) => ::std::option::Option::Some(_inner),
            Self::Unhandled(_inner) => ::std::option::Option::Some(&*_inner.source),
        }
    }
}
impl ::std::fmt::Display for DeleteConnectionGroupError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::AccessDenied(_inner) => _inner.fmt(f),
            Self::CannotDeleteEntityWhileInUse(_inner) => _inner.fmt(f),
            Self::EntityNotFound(_inner) => _inner.fmt(f),
            Self::InvalidIfMatchVersion(_inner) => _inner.fmt(f),
            Self::PreconditionFailed(_inner) => _inner.fmt(f),
            Self::ResourceNotDisabled(_inner) => _inner.fmt(f),
            Self::Unhandled(_inner) => {
                if let ::std::option::Option::Some(code) = ::aws_smithy_types::error::metadata::ProvideErrorMetadata::code(self) {
                    write!(f, "unhandled error ({code})")
                } else {
                    f.write_str("unhandled error")
                }
            }
        }
    }
}
impl ::aws_smithy_types::retry::ProvideErrorKind for DeleteConnectionGroupError {
    fn code(&self) -> ::std::option::Option<&str> {
        ::aws_smithy_types::error::metadata::ProvideErrorMetadata::code(self)
    }
    fn retryable_error_kind(&self) -> ::std::option::Option<::aws_smithy_types::retry::ErrorKind> {
        ::std::option::Option::None
    }
}
impl ::aws_smithy_types::error::metadata::ProvideErrorMetadata for DeleteConnectionGroupError {
    fn meta(&self) -> &::aws_smithy_types::error::ErrorMetadata {
        match self {
            Self::AccessDenied(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::CannotDeleteEntityWhileInUse(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::EntityNotFound(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::InvalidIfMatchVersion(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::PreconditionFailed(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::ResourceNotDisabled(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::Unhandled(_inner) => &_inner.meta,
        }
    }
}
impl ::aws_smithy_runtime_api::client::result::CreateUnhandledError for DeleteConnectionGroupError {
    fn create_unhandled_error(
        source: ::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static>,
        meta: ::std::option::Option<::aws_smithy_types::error::ErrorMetadata>,
    ) -> Self {
        Self::Unhandled(crate::error::sealed_unhandled::Unhandled {
            source,
            meta: meta.unwrap_or_default(),
        })
    }
}
impl ::aws_types::request_id::RequestId for crate::operation::delete_connection_group::DeleteConnectionGroupError {
    fn request_id(&self) -> Option<&str> {
        self.meta().request_id()
    }
}

pub use crate::operation::delete_connection_group::_delete_connection_group_output::DeleteConnectionGroupOutput;

pub use crate::operation::delete_connection_group::_delete_connection_group_input::DeleteConnectionGroupInput;

mod _delete_connection_group_input;

mod _delete_connection_group_output;

/// Builders
pub mod builders;

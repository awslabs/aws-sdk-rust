// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
/// Orchestration and serialization glue logic for `BatchGetDeploymentTargets`.
#[derive(::std::clone::Clone, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct BatchGetDeploymentTargets;
impl BatchGetDeploymentTargets {
    /// Creates a new `BatchGetDeploymentTargets`
    pub fn new() -> Self {
        Self
    }
    pub(crate) async fn orchestrate(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsInput,
    ) -> ::std::result::Result<
        crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let map_err = |err: ::aws_smithy_runtime_api::client::result::SdkError<
            ::aws_smithy_runtime_api::client::interceptors::context::Error,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >| {
            err.map_service_error(|err| {
                err.downcast::<crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsError>()
                    .expect("correct error type")
            })
        };
        let context = Self::orchestrate_with_stop_point(runtime_plugins, input, ::aws_smithy_runtime::client::orchestrator::StopPoint::None)
            .await
            .map_err(map_err)?;
        let output = context.finalize().map_err(map_err)?;
        ::std::result::Result::Ok(
            output
                .downcast::<crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsOutput>()
                .expect("correct output type"),
        )
    }

    pub(crate) async fn orchestrate_with_stop_point(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsInput,
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
        ::aws_smithy_runtime::client::orchestrator::invoke_with_stop_point(
            "CodeDeploy",
            "BatchGetDeploymentTargets",
            input,
            runtime_plugins,
            stop_point,
        )
        // Create a parent span for the entire operation. Includes a random, internal-only,
        // seven-digit ID for the operation orchestration so that it can be correlated in the logs.
        .instrument(::tracing::debug_span!(
            "CodeDeploy.BatchGetDeploymentTargets",
            "rpc.service" = "CodeDeploy",
            "rpc.method" = "BatchGetDeploymentTargets",
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
impl ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin for BatchGetDeploymentTargets {
    fn config(&self) -> ::std::option::Option<::aws_smithy_types::config_bag::FrozenLayer> {
        let mut cfg = ::aws_smithy_types::config_bag::Layer::new("BatchGetDeploymentTargets");

        cfg.store_put(::aws_smithy_runtime_api::client::ser_de::SharedRequestSerializer::new(
            BatchGetDeploymentTargetsRequestSerializer,
        ));
        cfg.store_put(::aws_smithy_runtime_api::client::ser_de::SharedResponseDeserializer::new(
            BatchGetDeploymentTargetsResponseDeserializer,
        ));

        cfg.store_put(::aws_smithy_runtime_api::client::auth::AuthSchemeOptionResolverParams::new(
            crate::config::auth::Params::builder()
                .operation_name("BatchGetDeploymentTargets")
                .build()
                .expect("required fields set"),
        ));

        cfg.store_put(::aws_smithy_runtime_api::client::orchestrator::Metadata::new(
            "BatchGetDeploymentTargets",
            "CodeDeploy",
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
        let mut rcb = ::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder::new("BatchGetDeploymentTargets")
            .with_interceptor(::aws_smithy_runtime::client::stalled_stream_protection::StalledStreamProtectionInterceptor::default())
            .with_interceptor(BatchGetDeploymentTargetsEndpointParamsInterceptor)
            .with_retry_classifier(::aws_smithy_runtime::client::retries::classifiers::TransientErrorClassifier::<
                crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsError,
            >::new())
            .with_retry_classifier(::aws_smithy_runtime::client::retries::classifiers::ModeledAsRetryableClassifier::<
                crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsError,
            >::new())
            .with_retry_classifier(::aws_runtime::retries::classifiers::AwsErrorCodeClassifier::<
                crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsError,
            >::new());

        ::std::borrow::Cow::Owned(rcb)
    }
}

#[derive(Debug)]
struct BatchGetDeploymentTargetsResponseDeserializer;
impl ::aws_smithy_runtime_api::client::ser_de::DeserializeResponse for BatchGetDeploymentTargetsResponseDeserializer {
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
        let parse_result = if !success && status != 200 || force_error {
            crate::protocol_serde::shape_batch_get_deployment_targets::de_batch_get_deployment_targets_http_error(status, headers, body)
        } else {
            crate::protocol_serde::shape_batch_get_deployment_targets::de_batch_get_deployment_targets_http_response(status, headers, body)
        };
        crate::protocol_serde::type_erase_result(parse_result)
    }
}
#[derive(Debug)]
struct BatchGetDeploymentTargetsRequestSerializer;
impl ::aws_smithy_runtime_api::client::ser_de::SerializeRequest for BatchGetDeploymentTargetsRequestSerializer {
    #[allow(unused_mut, clippy::let_and_return, clippy::needless_borrow, clippy::useless_conversion)]
    fn serialize_input(
        &self,
        input: ::aws_smithy_runtime_api::client::interceptors::context::Input,
        _cfg: &mut ::aws_smithy_types::config_bag::ConfigBag,
    ) -> ::std::result::Result<::aws_smithy_runtime_api::client::orchestrator::HttpRequest, ::aws_smithy_runtime_api::box_error::BoxError> {
        let input = input
            .downcast::<crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsInput>()
            .expect("correct type");
        let _header_serialization_settings = _cfg
            .load::<crate::serialization_settings::HeaderSerializationSettings>()
            .cloned()
            .unwrap_or_default();
        let mut request_builder = {
            fn uri_base(
                _input: &crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsInput,
                output: &mut ::std::string::String,
            ) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::BuildError> {
                use ::std::fmt::Write as _;
                ::std::write!(output, "/").expect("formatting should succeed");
                ::std::result::Result::Ok(())
            }
            #[allow(clippy::unnecessary_wraps)]
            fn update_http_builder(
                input: &crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsInput,
                builder: ::http::request::Builder,
            ) -> ::std::result::Result<::http::request::Builder, ::aws_smithy_types::error::operation::BuildError> {
                let mut uri = ::std::string::String::new();
                uri_base(input, &mut uri)?;
                ::std::result::Result::Ok(builder.method("POST").uri(uri))
            }
            let mut builder = update_http_builder(&input, ::http::request::Builder::new())?;
            builder = _header_serialization_settings.set_default_header(builder, ::http::header::CONTENT_TYPE, "application/x-amz-json-1.1");
            builder = _header_serialization_settings.set_default_header(
                builder,
                ::http::header::HeaderName::from_static("x-amz-target"),
                "CodeDeploy_20141006.BatchGetDeploymentTargets",
            );
            builder
        };
        let body = ::aws_smithy_types::body::SdkBody::from(
            crate::protocol_serde::shape_batch_get_deployment_targets::ser_batch_get_deployment_targets_input(&input)?,
        );
        if let Some(content_length) = body.content_length() {
            let content_length = content_length.to_string();
            request_builder = _header_serialization_settings.set_default_header(request_builder, ::http::header::CONTENT_LENGTH, &content_length);
        }
        ::std::result::Result::Ok(request_builder.body(body).expect("valid request").try_into().unwrap())
    }
}
#[derive(Debug)]
struct BatchGetDeploymentTargetsEndpointParamsInterceptor;

impl ::aws_smithy_runtime_api::client::interceptors::Intercept for BatchGetDeploymentTargetsEndpointParamsInterceptor {
    fn name(&self) -> &'static str {
        "BatchGetDeploymentTargetsEndpointParamsInterceptor"
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
            .downcast_ref::<BatchGetDeploymentTargetsInput>()
            .ok_or("failed to downcast to BatchGetDeploymentTargetsInput")?;

        let params = crate::config::endpoint::Params::builder()
            .set_region(cfg.load::<::aws_types::region::Region>().map(|r| r.as_ref().to_owned()))
            .set_use_dual_stack(cfg.load::<::aws_types::endpoint_config::UseDualStack>().map(|ty| ty.0))
            .set_use_fips(cfg.load::<::aws_types::endpoint_config::UseFips>().map(|ty| ty.0))
            .set_endpoint(cfg.load::<::aws_types::endpoint_config::EndpointUrl>().map(|ty| ty.0.clone()))
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

/// Error type for the `BatchGetDeploymentTargetsError` operation.
#[non_exhaustive]
#[derive(::std::fmt::Debug)]
pub enum BatchGetDeploymentTargetsError {
    /// <p>The deployment with the user or Amazon Web Services account does not exist.</p>
    DeploymentDoesNotExistException(crate::types::error::DeploymentDoesNotExistException),
    /// <p>At least one deployment ID must be specified.</p>
    DeploymentIdRequiredException(crate::types::error::DeploymentIdRequiredException),
    /// <p>The specified deployment has not started.</p>
    DeploymentNotStartedException(crate::types::error::DeploymentNotStartedException),
    /// <p>The provided target ID does not belong to the attempted deployment.</p>
    DeploymentTargetDoesNotExistException(crate::types::error::DeploymentTargetDoesNotExistException),
    /// <p>A deployment target ID was not provided.</p>
    DeploymentTargetIdRequiredException(crate::types::error::DeploymentTargetIdRequiredException),
    /// <p>The maximum number of targets that can be associated with an Amazon ECS or Lambda deployment was exceeded. The target list of both types of deployments must have exactly one item. This exception does not apply to EC2/On-premises deployments.</p>
    DeploymentTargetListSizeExceededException(crate::types::error::DeploymentTargetListSizeExceededException),
    /// <p>The specified instance does not exist in the deployment group.</p>
    #[deprecated(note = "This exception is deprecated, use DeploymentTargetDoesNotExistException instead.")]
    InstanceDoesNotExistException(crate::types::error::InstanceDoesNotExistException),
    /// <p>At least one of the deployment IDs was specified in an invalid format.</p>
    InvalidDeploymentIdException(crate::types::error::InvalidDeploymentIdException),
    /// <p>The target ID provided was not valid.</p>
    InvalidDeploymentTargetIdException(crate::types::error::InvalidDeploymentTargetIdException),
    /// An unexpected error occurred (e.g., invalid JSON returned by the service or an unknown error code).
    #[deprecated(note = "Matching `Unhandled` directly is not forwards compatible. Instead, match using a \
    variable wildcard pattern and check `.code()`:
     \
    &nbsp;&nbsp;&nbsp;`err if err.code() == Some(\"SpecificExceptionCode\") => { /* handle the error */ }`
     \
    See [`ProvideErrorMetadata`](#impl-ProvideErrorMetadata-for-BatchGetDeploymentTargetsError) for what information is available for the error.")]
    Unhandled(crate::error::sealed_unhandled::Unhandled),
}
impl BatchGetDeploymentTargetsError {
    /// Creates the `BatchGetDeploymentTargetsError::Unhandled` variant from any error type.
    pub fn unhandled(
        err: impl ::std::convert::Into<::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static>>,
    ) -> Self {
        Self::Unhandled(crate::error::sealed_unhandled::Unhandled {
            source: err.into(),
            meta: ::std::default::Default::default(),
        })
    }

    /// Creates the `BatchGetDeploymentTargetsError::Unhandled` variant from an [`ErrorMetadata`](::aws_smithy_types::error::ErrorMetadata).
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
            Self::DeploymentDoesNotExistException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::DeploymentIdRequiredException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::DeploymentNotStartedException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::DeploymentTargetDoesNotExistException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::DeploymentTargetIdRequiredException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::DeploymentTargetListSizeExceededException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::InstanceDoesNotExistException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::InvalidDeploymentIdException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::InvalidDeploymentTargetIdException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::Unhandled(e) => &e.meta,
        }
    }
    /// Returns `true` if the error kind is `BatchGetDeploymentTargetsError::DeploymentDoesNotExistException`.
    pub fn is_deployment_does_not_exist_exception(&self) -> bool {
        matches!(self, Self::DeploymentDoesNotExistException(_))
    }
    /// Returns `true` if the error kind is `BatchGetDeploymentTargetsError::DeploymentIdRequiredException`.
    pub fn is_deployment_id_required_exception(&self) -> bool {
        matches!(self, Self::DeploymentIdRequiredException(_))
    }
    /// Returns `true` if the error kind is `BatchGetDeploymentTargetsError::DeploymentNotStartedException`.
    pub fn is_deployment_not_started_exception(&self) -> bool {
        matches!(self, Self::DeploymentNotStartedException(_))
    }
    /// Returns `true` if the error kind is `BatchGetDeploymentTargetsError::DeploymentTargetDoesNotExistException`.
    pub fn is_deployment_target_does_not_exist_exception(&self) -> bool {
        matches!(self, Self::DeploymentTargetDoesNotExistException(_))
    }
    /// Returns `true` if the error kind is `BatchGetDeploymentTargetsError::DeploymentTargetIdRequiredException`.
    pub fn is_deployment_target_id_required_exception(&self) -> bool {
        matches!(self, Self::DeploymentTargetIdRequiredException(_))
    }
    /// Returns `true` if the error kind is `BatchGetDeploymentTargetsError::DeploymentTargetListSizeExceededException`.
    pub fn is_deployment_target_list_size_exceeded_exception(&self) -> bool {
        matches!(self, Self::DeploymentTargetListSizeExceededException(_))
    }
    /// Returns `true` if the error kind is `BatchGetDeploymentTargetsError::InstanceDoesNotExistException`.
    pub fn is_instance_does_not_exist_exception(&self) -> bool {
        matches!(self, Self::InstanceDoesNotExistException(_))
    }
    /// Returns `true` if the error kind is `BatchGetDeploymentTargetsError::InvalidDeploymentIdException`.
    pub fn is_invalid_deployment_id_exception(&self) -> bool {
        matches!(self, Self::InvalidDeploymentIdException(_))
    }
    /// Returns `true` if the error kind is `BatchGetDeploymentTargetsError::InvalidDeploymentTargetIdException`.
    pub fn is_invalid_deployment_target_id_exception(&self) -> bool {
        matches!(self, Self::InvalidDeploymentTargetIdException(_))
    }
}
impl ::std::error::Error for BatchGetDeploymentTargetsError {
    fn source(&self) -> ::std::option::Option<&(dyn ::std::error::Error + 'static)> {
        match self {
            Self::DeploymentDoesNotExistException(_inner) => ::std::option::Option::Some(_inner),
            Self::DeploymentIdRequiredException(_inner) => ::std::option::Option::Some(_inner),
            Self::DeploymentNotStartedException(_inner) => ::std::option::Option::Some(_inner),
            Self::DeploymentTargetDoesNotExistException(_inner) => ::std::option::Option::Some(_inner),
            Self::DeploymentTargetIdRequiredException(_inner) => ::std::option::Option::Some(_inner),
            Self::DeploymentTargetListSizeExceededException(_inner) => ::std::option::Option::Some(_inner),
            Self::InstanceDoesNotExistException(_inner) => ::std::option::Option::Some(_inner),
            Self::InvalidDeploymentIdException(_inner) => ::std::option::Option::Some(_inner),
            Self::InvalidDeploymentTargetIdException(_inner) => ::std::option::Option::Some(_inner),
            Self::Unhandled(_inner) => ::std::option::Option::Some(&*_inner.source),
        }
    }
}
impl ::std::fmt::Display for BatchGetDeploymentTargetsError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::DeploymentDoesNotExistException(_inner) => _inner.fmt(f),
            Self::DeploymentIdRequiredException(_inner) => _inner.fmt(f),
            Self::DeploymentNotStartedException(_inner) => _inner.fmt(f),
            Self::DeploymentTargetDoesNotExistException(_inner) => _inner.fmt(f),
            Self::DeploymentTargetIdRequiredException(_inner) => _inner.fmt(f),
            Self::DeploymentTargetListSizeExceededException(_inner) => _inner.fmt(f),
            Self::InstanceDoesNotExistException(_inner) => _inner.fmt(f),
            Self::InvalidDeploymentIdException(_inner) => _inner.fmt(f),
            Self::InvalidDeploymentTargetIdException(_inner) => _inner.fmt(f),
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
impl ::aws_smithy_types::retry::ProvideErrorKind for BatchGetDeploymentTargetsError {
    fn code(&self) -> ::std::option::Option<&str> {
        ::aws_smithy_types::error::metadata::ProvideErrorMetadata::code(self)
    }
    fn retryable_error_kind(&self) -> ::std::option::Option<::aws_smithy_types::retry::ErrorKind> {
        ::std::option::Option::None
    }
}
impl ::aws_smithy_types::error::metadata::ProvideErrorMetadata for BatchGetDeploymentTargetsError {
    fn meta(&self) -> &::aws_smithy_types::error::ErrorMetadata {
        match self {
            Self::DeploymentDoesNotExistException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::DeploymentIdRequiredException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::DeploymentNotStartedException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::DeploymentTargetDoesNotExistException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::DeploymentTargetIdRequiredException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::DeploymentTargetListSizeExceededException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::InstanceDoesNotExistException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::InvalidDeploymentIdException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::InvalidDeploymentTargetIdException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::Unhandled(_inner) => &_inner.meta,
        }
    }
}
impl ::aws_smithy_runtime_api::client::result::CreateUnhandledError for BatchGetDeploymentTargetsError {
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
impl ::aws_types::request_id::RequestId for crate::operation::batch_get_deployment_targets::BatchGetDeploymentTargetsError {
    fn request_id(&self) -> Option<&str> {
        self.meta().request_id()
    }
}

pub use crate::operation::batch_get_deployment_targets::_batch_get_deployment_targets_output::BatchGetDeploymentTargetsOutput;

pub use crate::operation::batch_get_deployment_targets::_batch_get_deployment_targets_input::BatchGetDeploymentTargetsInput;

mod _batch_get_deployment_targets_input;

mod _batch_get_deployment_targets_output;

/// Builders
pub mod builders;

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
/// Orchestration and serialization glue logic for `EnableSnapshotCopy`.
#[derive(::std::clone::Clone, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct EnableSnapshotCopy;
impl EnableSnapshotCopy {
    /// Creates a new `EnableSnapshotCopy`
    pub fn new() -> Self {
        Self
    }
    pub(crate) async fn orchestrate(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::enable_snapshot_copy::EnableSnapshotCopyInput,
    ) -> ::std::result::Result<
        crate::operation::enable_snapshot_copy::EnableSnapshotCopyOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::enable_snapshot_copy::EnableSnapshotCopyError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let map_err = |err: ::aws_smithy_runtime_api::client::result::SdkError<
            ::aws_smithy_runtime_api::client::interceptors::context::Error,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >| {
            err.map_service_error(|err| {
                err.downcast::<crate::operation::enable_snapshot_copy::EnableSnapshotCopyError>()
                    .expect("correct error type")
            })
        };
        let context = Self::orchestrate_with_stop_point(runtime_plugins, input, ::aws_smithy_runtime::client::orchestrator::StopPoint::None)
            .await
            .map_err(map_err)?;
        let output = context.finalize().map_err(map_err)?;
        ::std::result::Result::Ok(
            output
                .downcast::<crate::operation::enable_snapshot_copy::EnableSnapshotCopyOutput>()
                .expect("correct output type"),
        )
    }

    pub(crate) async fn orchestrate_with_stop_point(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::enable_snapshot_copy::EnableSnapshotCopyInput,
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
        ::aws_smithy_runtime::client::orchestrator::invoke_with_stop_point("Redshift", "EnableSnapshotCopy", input, runtime_plugins, stop_point)
            // Create a parent span for the entire operation. Includes a random, internal-only,
            // seven-digit ID for the operation orchestration so that it can be correlated in the logs.
            .instrument(::tracing::debug_span!(
                "Redshift.EnableSnapshotCopy",
                "rpc.service" = "Redshift",
                "rpc.method" = "EnableSnapshotCopy",
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
impl ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin for EnableSnapshotCopy {
    fn config(&self) -> ::std::option::Option<::aws_smithy_types::config_bag::FrozenLayer> {
        let mut cfg = ::aws_smithy_types::config_bag::Layer::new("EnableSnapshotCopy");

        cfg.store_put(::aws_smithy_runtime_api::client::ser_de::SharedRequestSerializer::new(
            EnableSnapshotCopyRequestSerializer,
        ));
        cfg.store_put(::aws_smithy_runtime_api::client::ser_de::SharedResponseDeserializer::new(
            EnableSnapshotCopyResponseDeserializer,
        ));

        cfg.store_put(::aws_smithy_runtime_api::client::auth::AuthSchemeOptionResolverParams::new(
            crate::config::auth::Params::builder()
                .operation_name("EnableSnapshotCopy")
                .build()
                .expect("required fields set"),
        ));

        cfg.store_put(::aws_smithy_runtime_api::client::orchestrator::SensitiveOutput);
        cfg.store_put(::aws_smithy_runtime_api::client::orchestrator::Metadata::new(
            "EnableSnapshotCopy",
            "Redshift",
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
        let mut rcb = ::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder::new("EnableSnapshotCopy")
            .with_interceptor(::aws_smithy_runtime::client::stalled_stream_protection::StalledStreamProtectionInterceptor::default())
            .with_interceptor(EnableSnapshotCopyEndpointParamsInterceptor)
            .with_retry_classifier(::aws_smithy_runtime::client::retries::classifiers::TransientErrorClassifier::<
                crate::operation::enable_snapshot_copy::EnableSnapshotCopyError,
            >::new())
            .with_retry_classifier(::aws_smithy_runtime::client::retries::classifiers::ModeledAsRetryableClassifier::<
                crate::operation::enable_snapshot_copy::EnableSnapshotCopyError,
            >::new())
            .with_retry_classifier(::aws_runtime::retries::classifiers::AwsErrorCodeClassifier::<
                crate::operation::enable_snapshot_copy::EnableSnapshotCopyError,
            >::new());

        ::std::borrow::Cow::Owned(rcb)
    }
}

#[derive(Debug)]
struct EnableSnapshotCopyResponseDeserializer;
impl ::aws_smithy_runtime_api::client::ser_de::DeserializeResponse for EnableSnapshotCopyResponseDeserializer {
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
            crate::protocol_serde::shape_enable_snapshot_copy::de_enable_snapshot_copy_http_error(status, headers, body)
        } else {
            crate::protocol_serde::shape_enable_snapshot_copy::de_enable_snapshot_copy_http_response(status, headers, body)
        };
        crate::protocol_serde::type_erase_result(parse_result)
    }
}
#[derive(Debug)]
struct EnableSnapshotCopyRequestSerializer;
impl ::aws_smithy_runtime_api::client::ser_de::SerializeRequest for EnableSnapshotCopyRequestSerializer {
    #[allow(unused_mut, clippy::let_and_return, clippy::needless_borrow, clippy::useless_conversion)]
    fn serialize_input(
        &self,
        input: ::aws_smithy_runtime_api::client::interceptors::context::Input,
        _cfg: &mut ::aws_smithy_types::config_bag::ConfigBag,
    ) -> ::std::result::Result<::aws_smithy_runtime_api::client::orchestrator::HttpRequest, ::aws_smithy_runtime_api::box_error::BoxError> {
        let input = input
            .downcast::<crate::operation::enable_snapshot_copy::EnableSnapshotCopyInput>()
            .expect("correct type");
        let _header_serialization_settings = _cfg
            .load::<crate::serialization_settings::HeaderSerializationSettings>()
            .cloned()
            .unwrap_or_default();
        let mut request_builder = {
            fn uri_base(
                _input: &crate::operation::enable_snapshot_copy::EnableSnapshotCopyInput,
                output: &mut ::std::string::String,
            ) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::BuildError> {
                use ::std::fmt::Write as _;
                ::std::write!(output, "/").expect("formatting should succeed");
                ::std::result::Result::Ok(())
            }
            #[allow(clippy::unnecessary_wraps)]
            fn update_http_builder(
                input: &crate::operation::enable_snapshot_copy::EnableSnapshotCopyInput,
                builder: ::http::request::Builder,
            ) -> ::std::result::Result<::http::request::Builder, ::aws_smithy_types::error::operation::BuildError> {
                let mut uri = ::std::string::String::new();
                uri_base(input, &mut uri)?;
                ::std::result::Result::Ok(builder.method("POST").uri(uri))
            }
            let mut builder = update_http_builder(&input, ::http::request::Builder::new())?;
            builder = _header_serialization_settings.set_default_header(builder, ::http::header::CONTENT_TYPE, "application/x-www-form-urlencoded");
            builder
        };
        let body = ::aws_smithy_types::body::SdkBody::from(
            crate::protocol_serde::shape_enable_snapshot_copy_input::ser_enable_snapshot_copy_input_input_input(&input)?,
        );
        if let Some(content_length) = body.content_length() {
            let content_length = content_length.to_string();
            request_builder = _header_serialization_settings.set_default_header(request_builder, ::http::header::CONTENT_LENGTH, &content_length);
        }
        ::std::result::Result::Ok(request_builder.body(body).expect("valid request").try_into().unwrap())
    }
}
#[derive(Debug)]
struct EnableSnapshotCopyEndpointParamsInterceptor;

impl ::aws_smithy_runtime_api::client::interceptors::Intercept for EnableSnapshotCopyEndpointParamsInterceptor {
    fn name(&self) -> &'static str {
        "EnableSnapshotCopyEndpointParamsInterceptor"
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
            .downcast_ref::<EnableSnapshotCopyInput>()
            .ok_or("failed to downcast to EnableSnapshotCopyInput")?;

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

/// Error type for the `EnableSnapshotCopyError` operation.
#[non_exhaustive]
#[derive(::std::fmt::Debug)]
pub enum EnableSnapshotCopyError {
    /// <p>The <code>ClusterIdentifier</code> parameter does not refer to an existing cluster.</p>
    ClusterNotFoundFault(crate::types::error::ClusterNotFoundFault),
    /// <p>Cross-region snapshot copy was temporarily disabled. Try your request again.</p>
    CopyToRegionDisabledFault(crate::types::error::CopyToRegionDisabledFault),
    /// <p>The request cannot be completed because a dependent service is throttling requests made by Amazon Redshift on your behalf. Wait and retry the request.</p>
    DependentServiceRequestThrottlingFault(crate::types::error::DependentServiceRequestThrottlingFault),
    /// <p>The specified options are incompatible.</p>
    IncompatibleOrderableOptions(crate::types::error::IncompatibleOrderableOptions),
    /// <p>The specified cluster is not in the <code>available</code> state.</p>
    InvalidClusterStateFault(crate::types::error::InvalidClusterStateFault),
    /// <p>The retention period specified is either in the past or is not a valid value.</p>
    /// <p>The value must be either -1 or an integer between 1 and 3,653.</p>
    InvalidRetentionPeriodFault(crate::types::error::InvalidRetentionPeriodFault),
    /// <p>The encryption key has exceeded its grant limit in Amazon Web Services KMS.</p>
    LimitExceededFault(crate::types::error::LimitExceededFault),
    /// <p>The cluster already has cross-region snapshot copy enabled.</p>
    SnapshotCopyAlreadyEnabledFault(crate::types::error::SnapshotCopyAlreadyEnabledFault),
    /// <p>The specified snapshot copy grant can't be found. Make sure that the name is typed correctly and that the grant exists in the destination region.</p>
    SnapshotCopyGrantNotFoundFault(crate::types::error::SnapshotCopyGrantNotFoundFault),
    /// <p>Your account is not authorized to perform the requested operation.</p>
    UnauthorizedOperation(crate::types::error::UnauthorizedOperation),
    /// <p>The specified region is incorrect or does not exist.</p>
    UnknownSnapshotCopyRegionFault(crate::types::error::UnknownSnapshotCopyRegionFault),
    /// An unexpected error occurred (e.g., invalid JSON returned by the service or an unknown error code).
    #[deprecated(note = "Matching `Unhandled` directly is not forwards compatible. Instead, match using a \
    variable wildcard pattern and check `.code()`:
     \
    &nbsp;&nbsp;&nbsp;`err if err.code() == Some(\"SpecificExceptionCode\") => { /* handle the error */ }`
     \
    See [`ProvideErrorMetadata`](#impl-ProvideErrorMetadata-for-EnableSnapshotCopyError) for what information is available for the error.")]
    Unhandled(crate::error::sealed_unhandled::Unhandled),
}
impl EnableSnapshotCopyError {
    /// Creates the `EnableSnapshotCopyError::Unhandled` variant from any error type.
    pub fn unhandled(
        err: impl ::std::convert::Into<::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static>>,
    ) -> Self {
        Self::Unhandled(crate::error::sealed_unhandled::Unhandled {
            source: err.into(),
            meta: ::std::default::Default::default(),
        })
    }

    /// Creates the `EnableSnapshotCopyError::Unhandled` variant from an [`ErrorMetadata`](::aws_smithy_types::error::ErrorMetadata).
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
            Self::ClusterNotFoundFault(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::CopyToRegionDisabledFault(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::DependentServiceRequestThrottlingFault(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::IncompatibleOrderableOptions(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::InvalidClusterStateFault(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::InvalidRetentionPeriodFault(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::LimitExceededFault(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::SnapshotCopyAlreadyEnabledFault(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::SnapshotCopyGrantNotFoundFault(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::UnauthorizedOperation(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::UnknownSnapshotCopyRegionFault(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::Unhandled(e) => &e.meta,
        }
    }
    /// Returns `true` if the error kind is `EnableSnapshotCopyError::ClusterNotFoundFault`.
    pub fn is_cluster_not_found_fault(&self) -> bool {
        matches!(self, Self::ClusterNotFoundFault(_))
    }
    /// Returns `true` if the error kind is `EnableSnapshotCopyError::CopyToRegionDisabledFault`.
    pub fn is_copy_to_region_disabled_fault(&self) -> bool {
        matches!(self, Self::CopyToRegionDisabledFault(_))
    }
    /// Returns `true` if the error kind is `EnableSnapshotCopyError::DependentServiceRequestThrottlingFault`.
    pub fn is_dependent_service_request_throttling_fault(&self) -> bool {
        matches!(self, Self::DependentServiceRequestThrottlingFault(_))
    }
    /// Returns `true` if the error kind is `EnableSnapshotCopyError::IncompatibleOrderableOptions`.
    pub fn is_incompatible_orderable_options(&self) -> bool {
        matches!(self, Self::IncompatibleOrderableOptions(_))
    }
    /// Returns `true` if the error kind is `EnableSnapshotCopyError::InvalidClusterStateFault`.
    pub fn is_invalid_cluster_state_fault(&self) -> bool {
        matches!(self, Self::InvalidClusterStateFault(_))
    }
    /// Returns `true` if the error kind is `EnableSnapshotCopyError::InvalidRetentionPeriodFault`.
    pub fn is_invalid_retention_period_fault(&self) -> bool {
        matches!(self, Self::InvalidRetentionPeriodFault(_))
    }
    /// Returns `true` if the error kind is `EnableSnapshotCopyError::LimitExceededFault`.
    pub fn is_limit_exceeded_fault(&self) -> bool {
        matches!(self, Self::LimitExceededFault(_))
    }
    /// Returns `true` if the error kind is `EnableSnapshotCopyError::SnapshotCopyAlreadyEnabledFault`.
    pub fn is_snapshot_copy_already_enabled_fault(&self) -> bool {
        matches!(self, Self::SnapshotCopyAlreadyEnabledFault(_))
    }
    /// Returns `true` if the error kind is `EnableSnapshotCopyError::SnapshotCopyGrantNotFoundFault`.
    pub fn is_snapshot_copy_grant_not_found_fault(&self) -> bool {
        matches!(self, Self::SnapshotCopyGrantNotFoundFault(_))
    }
    /// Returns `true` if the error kind is `EnableSnapshotCopyError::UnauthorizedOperation`.
    pub fn is_unauthorized_operation(&self) -> bool {
        matches!(self, Self::UnauthorizedOperation(_))
    }
    /// Returns `true` if the error kind is `EnableSnapshotCopyError::UnknownSnapshotCopyRegionFault`.
    pub fn is_unknown_snapshot_copy_region_fault(&self) -> bool {
        matches!(self, Self::UnknownSnapshotCopyRegionFault(_))
    }
}
impl ::std::error::Error for EnableSnapshotCopyError {
    fn source(&self) -> ::std::option::Option<&(dyn ::std::error::Error + 'static)> {
        match self {
            Self::ClusterNotFoundFault(_inner) => ::std::option::Option::Some(_inner),
            Self::CopyToRegionDisabledFault(_inner) => ::std::option::Option::Some(_inner),
            Self::DependentServiceRequestThrottlingFault(_inner) => ::std::option::Option::Some(_inner),
            Self::IncompatibleOrderableOptions(_inner) => ::std::option::Option::Some(_inner),
            Self::InvalidClusterStateFault(_inner) => ::std::option::Option::Some(_inner),
            Self::InvalidRetentionPeriodFault(_inner) => ::std::option::Option::Some(_inner),
            Self::LimitExceededFault(_inner) => ::std::option::Option::Some(_inner),
            Self::SnapshotCopyAlreadyEnabledFault(_inner) => ::std::option::Option::Some(_inner),
            Self::SnapshotCopyGrantNotFoundFault(_inner) => ::std::option::Option::Some(_inner),
            Self::UnauthorizedOperation(_inner) => ::std::option::Option::Some(_inner),
            Self::UnknownSnapshotCopyRegionFault(_inner) => ::std::option::Option::Some(_inner),
            Self::Unhandled(_inner) => ::std::option::Option::Some(&*_inner.source),
        }
    }
}
impl ::std::fmt::Display for EnableSnapshotCopyError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::ClusterNotFoundFault(_inner) => _inner.fmt(f),
            Self::CopyToRegionDisabledFault(_inner) => _inner.fmt(f),
            Self::DependentServiceRequestThrottlingFault(_inner) => _inner.fmt(f),
            Self::IncompatibleOrderableOptions(_inner) => _inner.fmt(f),
            Self::InvalidClusterStateFault(_inner) => _inner.fmt(f),
            Self::InvalidRetentionPeriodFault(_inner) => _inner.fmt(f),
            Self::LimitExceededFault(_inner) => _inner.fmt(f),
            Self::SnapshotCopyAlreadyEnabledFault(_inner) => _inner.fmt(f),
            Self::SnapshotCopyGrantNotFoundFault(_inner) => _inner.fmt(f),
            Self::UnauthorizedOperation(_inner) => _inner.fmt(f),
            Self::UnknownSnapshotCopyRegionFault(_inner) => _inner.fmt(f),
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
impl ::aws_smithy_types::retry::ProvideErrorKind for EnableSnapshotCopyError {
    fn code(&self) -> ::std::option::Option<&str> {
        ::aws_smithy_types::error::metadata::ProvideErrorMetadata::code(self)
    }
    fn retryable_error_kind(&self) -> ::std::option::Option<::aws_smithy_types::retry::ErrorKind> {
        ::std::option::Option::None
    }
}
impl ::aws_smithy_types::error::metadata::ProvideErrorMetadata for EnableSnapshotCopyError {
    fn meta(&self) -> &::aws_smithy_types::error::ErrorMetadata {
        match self {
            Self::ClusterNotFoundFault(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::CopyToRegionDisabledFault(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::DependentServiceRequestThrottlingFault(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::IncompatibleOrderableOptions(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::InvalidClusterStateFault(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::InvalidRetentionPeriodFault(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::LimitExceededFault(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::SnapshotCopyAlreadyEnabledFault(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::SnapshotCopyGrantNotFoundFault(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::UnauthorizedOperation(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::UnknownSnapshotCopyRegionFault(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::Unhandled(_inner) => &_inner.meta,
        }
    }
}
impl ::aws_smithy_runtime_api::client::result::CreateUnhandledError for EnableSnapshotCopyError {
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
impl ::aws_types::request_id::RequestId for crate::operation::enable_snapshot_copy::EnableSnapshotCopyError {
    fn request_id(&self) -> Option<&str> {
        self.meta().request_id()
    }
}

pub use crate::operation::enable_snapshot_copy::_enable_snapshot_copy_output::EnableSnapshotCopyOutput;

pub use crate::operation::enable_snapshot_copy::_enable_snapshot_copy_input::EnableSnapshotCopyInput;

mod _enable_snapshot_copy_input;

mod _enable_snapshot_copy_output;

/// Builders
pub mod builders;

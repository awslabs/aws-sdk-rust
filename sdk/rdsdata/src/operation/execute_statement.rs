// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
/// Orchestration and serialization glue logic for `ExecuteStatement`.
#[derive(::std::clone::Clone, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct ExecuteStatement;
impl ExecuteStatement {
    /// Creates a new `ExecuteStatement`
    pub fn new() -> Self {
        Self
    }
    pub(crate) async fn orchestrate(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::execute_statement::ExecuteStatementInput,
    ) -> ::std::result::Result<
        crate::operation::execute_statement::ExecuteStatementOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::execute_statement::ExecuteStatementError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let map_err = |err: ::aws_smithy_runtime_api::client::result::SdkError<
            ::aws_smithy_runtime_api::client::interceptors::context::Error,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >| {
            err.map_service_error(|err| {
                err.downcast::<crate::operation::execute_statement::ExecuteStatementError>()
                    .expect("correct error type")
            })
        };
        let context = Self::orchestrate_with_stop_point(runtime_plugins, input, ::aws_smithy_runtime::client::orchestrator::StopPoint::None)
            .await
            .map_err(map_err)?;
        let output = context.finalize().map_err(map_err)?;
        ::std::result::Result::Ok(
            output
                .downcast::<crate::operation::execute_statement::ExecuteStatementOutput>()
                .expect("correct output type"),
        )
    }

    pub(crate) async fn orchestrate_with_stop_point(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::execute_statement::ExecuteStatementInput,
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
        ::aws_smithy_runtime::client::orchestrator::invoke_with_stop_point("RDS Data", "ExecuteStatement", input, runtime_plugins, stop_point)
            // Create a parent span for the entire operation. Includes a random, internal-only,
            // seven-digit ID for the operation orchestration so that it can be correlated in the logs.
            .instrument(::tracing::debug_span!(
                "RDS Data.ExecuteStatement",
                "rpc.service" = "RDS Data",
                "rpc.method" = "ExecuteStatement",
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
impl ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin for ExecuteStatement {
    fn config(&self) -> ::std::option::Option<::aws_smithy_types::config_bag::FrozenLayer> {
        let mut cfg = ::aws_smithy_types::config_bag::Layer::new("ExecuteStatement");

        cfg.store_put(::aws_smithy_runtime_api::client::ser_de::SharedRequestSerializer::new(
            ExecuteStatementRequestSerializer,
        ));
        cfg.store_put(::aws_smithy_runtime_api::client::ser_de::SharedResponseDeserializer::new(
            ExecuteStatementResponseDeserializer,
        ));

        cfg.store_put(::aws_smithy_runtime_api::client::auth::AuthSchemeOptionResolverParams::new(
            crate::config::auth::Params::builder()
                .operation_name("ExecuteStatement")
                .build()
                .expect("required fields set"),
        ));

        cfg.store_put(::aws_smithy_runtime_api::client::orchestrator::Metadata::new(
            "ExecuteStatement",
            "RDS Data",
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
        let mut rcb = ::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder::new("ExecuteStatement")
            .with_interceptor(::aws_smithy_runtime::client::stalled_stream_protection::StalledStreamProtectionInterceptor::default())
            .with_interceptor(ExecuteStatementEndpointParamsInterceptor)
            .with_retry_classifier(::aws_smithy_runtime::client::retries::classifiers::TransientErrorClassifier::<
                crate::operation::execute_statement::ExecuteStatementError,
            >::new())
            .with_retry_classifier(::aws_smithy_runtime::client::retries::classifiers::ModeledAsRetryableClassifier::<
                crate::operation::execute_statement::ExecuteStatementError,
            >::new())
            .with_retry_classifier(::aws_runtime::retries::classifiers::AwsErrorCodeClassifier::<
                crate::operation::execute_statement::ExecuteStatementError,
            >::new());

        ::std::borrow::Cow::Owned(rcb)
    }
}

#[derive(Debug)]
struct ExecuteStatementResponseDeserializer;
impl ::aws_smithy_runtime_api::client::ser_de::DeserializeResponse for ExecuteStatementResponseDeserializer {
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
            crate::protocol_serde::shape_execute_statement::de_execute_statement_http_error(status, headers, body)
        } else {
            crate::protocol_serde::shape_execute_statement::de_execute_statement_http_response(status, headers, body)
        };
        crate::protocol_serde::type_erase_result(parse_result)
    }
}
#[derive(Debug)]
struct ExecuteStatementRequestSerializer;
impl ::aws_smithy_runtime_api::client::ser_de::SerializeRequest for ExecuteStatementRequestSerializer {
    #[allow(unused_mut, clippy::let_and_return, clippy::needless_borrow, clippy::useless_conversion)]
    fn serialize_input(
        &self,
        input: ::aws_smithy_runtime_api::client::interceptors::context::Input,
        _cfg: &mut ::aws_smithy_types::config_bag::ConfigBag,
    ) -> ::std::result::Result<::aws_smithy_runtime_api::client::orchestrator::HttpRequest, ::aws_smithy_runtime_api::box_error::BoxError> {
        let input = input
            .downcast::<crate::operation::execute_statement::ExecuteStatementInput>()
            .expect("correct type");
        let _header_serialization_settings = _cfg
            .load::<crate::serialization_settings::HeaderSerializationSettings>()
            .cloned()
            .unwrap_or_default();
        let mut request_builder = {
            fn uri_base(
                _input: &crate::operation::execute_statement::ExecuteStatementInput,
                output: &mut ::std::string::String,
            ) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::BuildError> {
                use ::std::fmt::Write as _;
                ::std::write!(output, "/Execute").expect("formatting should succeed");
                ::std::result::Result::Ok(())
            }
            #[allow(clippy::unnecessary_wraps)]
            fn update_http_builder(
                input: &crate::operation::execute_statement::ExecuteStatementInput,
                builder: ::http::request::Builder,
            ) -> ::std::result::Result<::http::request::Builder, ::aws_smithy_types::error::operation::BuildError> {
                let mut uri = ::std::string::String::new();
                uri_base(input, &mut uri)?;
                ::std::result::Result::Ok(builder.method("POST").uri(uri))
            }
            let mut builder = update_http_builder(&input, ::http::request::Builder::new())?;
            builder = _header_serialization_settings.set_default_header(builder, ::http::header::CONTENT_TYPE, "application/json");
            builder
        };
        let body = ::aws_smithy_types::body::SdkBody::from(crate::protocol_serde::shape_execute_statement::ser_execute_statement_input(&input)?);
        if let Some(content_length) = body.content_length() {
            let content_length = content_length.to_string();
            request_builder = _header_serialization_settings.set_default_header(request_builder, ::http::header::CONTENT_LENGTH, &content_length);
        }
        ::std::result::Result::Ok(request_builder.body(body).expect("valid request").try_into().unwrap())
    }
}
#[derive(Debug)]
struct ExecuteStatementEndpointParamsInterceptor;

impl ::aws_smithy_runtime_api::client::interceptors::Intercept for ExecuteStatementEndpointParamsInterceptor {
    fn name(&self) -> &'static str {
        "ExecuteStatementEndpointParamsInterceptor"
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
            .downcast_ref::<ExecuteStatementInput>()
            .ok_or("failed to downcast to ExecuteStatementInput")?;

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

/// Error type for the `ExecuteStatementError` operation.
#[non_exhaustive]
#[derive(::std::fmt::Debug)]
pub enum ExecuteStatementError {
    /// <p>You don't have sufficient access to perform this action.</p>
    AccessDeniedException(crate::types::error::AccessDeniedException),
    /// <p>There is an error in the call or in a SQL statement. (This error only appears in calls from Aurora Serverless v1 databases.)</p>
    BadRequestException(crate::types::error::BadRequestException),
    /// <p>There was an error in processing the SQL statement.</p>
    DatabaseErrorException(crate::types::error::DatabaseErrorException),
    /// <p>The DB cluster doesn't have a DB instance.</p>
    DatabaseNotFoundException(crate::types::error::DatabaseNotFoundException),
    /// <p>A request was cancelled because the Aurora Serverless v2 DB instance was paused. The Data API request automatically resumes the DB instance. Wait a few seconds and try again.</p>
    DatabaseResumingException(crate::types::error::DatabaseResumingException),
    /// <p>The writer instance in the DB cluster isn't available.</p>
    DatabaseUnavailableException(crate::types::error::DatabaseUnavailableException),
    /// <p>There are insufficient privileges to make the call.</p>
    ForbiddenException(crate::types::error::ForbiddenException),
    /// <p>The HTTP endpoint for using RDS Data API isn't enabled for the DB cluster.</p>
    HttpEndpointNotEnabledException(crate::types::error::HttpEndpointNotEnabledException),
    /// <p>An internal error occurred.</p>
    InternalServerErrorException(crate::types::error::InternalServerErrorException),
    /// <p>The resource is in an invalid state.</p>
    InvalidResourceStateException(crate::types::error::InvalidResourceStateException),
    /// <p>The Secrets Manager secret used with the request isn't valid.</p>
    InvalidSecretException(crate::types::error::InvalidSecretException),
    /// <p>There was a problem with the Secrets Manager secret used with the request, caused by one of the following conditions:</p>
    /// <ul>
    /// <li>
    /// <p>RDS Data API timed out retrieving the secret.</p></li>
    /// <li>
    /// <p>The secret provided wasn't found.</p></li>
    /// <li>
    /// <p>The secret couldn't be decrypted.</p></li>
    /// </ul>
    SecretsErrorException(crate::types::error::SecretsErrorException),
    /// <p>The service specified by the <code>resourceArn</code> parameter isn't available.</p>
    ServiceUnavailableError(crate::types::error::ServiceUnavailableError),
    /// <p>The execution of the SQL statement timed out.</p>
    StatementTimeoutException(crate::types::error::StatementTimeoutException),
    /// <p>The transaction ID wasn't found.</p>
    TransactionNotFoundException(crate::types::error::TransactionNotFoundException),
    /// <p>There was a problem with the result because of one of the following conditions:</p>
    /// <ul>
    /// <li>
    /// <p>It contained an unsupported data type.</p></li>
    /// <li>
    /// <p>It contained a multidimensional array.</p></li>
    /// <li>
    /// <p>The size was too large.</p></li>
    /// </ul>
    UnsupportedResultException(crate::types::error::UnsupportedResultException),
    /// An unexpected error occurred (e.g., invalid JSON returned by the service or an unknown error code).
    #[deprecated(note = "Matching `Unhandled` directly is not forwards compatible. Instead, match using a \
    variable wildcard pattern and check `.code()`:
     \
    &nbsp;&nbsp;&nbsp;`err if err.code() == Some(\"SpecificExceptionCode\") => { /* handle the error */ }`
     \
    See [`ProvideErrorMetadata`](#impl-ProvideErrorMetadata-for-ExecuteStatementError) for what information is available for the error.")]
    Unhandled(crate::error::sealed_unhandled::Unhandled),
}
impl ExecuteStatementError {
    /// Creates the `ExecuteStatementError::Unhandled` variant from any error type.
    pub fn unhandled(
        err: impl ::std::convert::Into<::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static>>,
    ) -> Self {
        Self::Unhandled(crate::error::sealed_unhandled::Unhandled {
            source: err.into(),
            meta: ::std::default::Default::default(),
        })
    }

    /// Creates the `ExecuteStatementError::Unhandled` variant from an [`ErrorMetadata`](::aws_smithy_types::error::ErrorMetadata).
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
            Self::AccessDeniedException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::BadRequestException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::DatabaseErrorException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::DatabaseNotFoundException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::DatabaseResumingException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::DatabaseUnavailableException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::ForbiddenException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::HttpEndpointNotEnabledException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::InternalServerErrorException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::InvalidResourceStateException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::InvalidSecretException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::SecretsErrorException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::ServiceUnavailableError(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::StatementTimeoutException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::TransactionNotFoundException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::UnsupportedResultException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::Unhandled(e) => &e.meta,
        }
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::AccessDeniedException`.
    pub fn is_access_denied_exception(&self) -> bool {
        matches!(self, Self::AccessDeniedException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::BadRequestException`.
    pub fn is_bad_request_exception(&self) -> bool {
        matches!(self, Self::BadRequestException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::DatabaseErrorException`.
    pub fn is_database_error_exception(&self) -> bool {
        matches!(self, Self::DatabaseErrorException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::DatabaseNotFoundException`.
    pub fn is_database_not_found_exception(&self) -> bool {
        matches!(self, Self::DatabaseNotFoundException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::DatabaseResumingException`.
    pub fn is_database_resuming_exception(&self) -> bool {
        matches!(self, Self::DatabaseResumingException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::DatabaseUnavailableException`.
    pub fn is_database_unavailable_exception(&self) -> bool {
        matches!(self, Self::DatabaseUnavailableException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::ForbiddenException`.
    pub fn is_forbidden_exception(&self) -> bool {
        matches!(self, Self::ForbiddenException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::HttpEndpointNotEnabledException`.
    pub fn is_http_endpoint_not_enabled_exception(&self) -> bool {
        matches!(self, Self::HttpEndpointNotEnabledException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::InternalServerErrorException`.
    pub fn is_internal_server_error_exception(&self) -> bool {
        matches!(self, Self::InternalServerErrorException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::InvalidResourceStateException`.
    pub fn is_invalid_resource_state_exception(&self) -> bool {
        matches!(self, Self::InvalidResourceStateException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::InvalidSecretException`.
    pub fn is_invalid_secret_exception(&self) -> bool {
        matches!(self, Self::InvalidSecretException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::SecretsErrorException`.
    pub fn is_secrets_error_exception(&self) -> bool {
        matches!(self, Self::SecretsErrorException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::ServiceUnavailableError`.
    pub fn is_service_unavailable_error(&self) -> bool {
        matches!(self, Self::ServiceUnavailableError(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::StatementTimeoutException`.
    pub fn is_statement_timeout_exception(&self) -> bool {
        matches!(self, Self::StatementTimeoutException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::TransactionNotFoundException`.
    pub fn is_transaction_not_found_exception(&self) -> bool {
        matches!(self, Self::TransactionNotFoundException(_))
    }
    /// Returns `true` if the error kind is `ExecuteStatementError::UnsupportedResultException`.
    pub fn is_unsupported_result_exception(&self) -> bool {
        matches!(self, Self::UnsupportedResultException(_))
    }
}
impl ::std::error::Error for ExecuteStatementError {
    fn source(&self) -> ::std::option::Option<&(dyn ::std::error::Error + 'static)> {
        match self {
            Self::AccessDeniedException(_inner) => ::std::option::Option::Some(_inner),
            Self::BadRequestException(_inner) => ::std::option::Option::Some(_inner),
            Self::DatabaseErrorException(_inner) => ::std::option::Option::Some(_inner),
            Self::DatabaseNotFoundException(_inner) => ::std::option::Option::Some(_inner),
            Self::DatabaseResumingException(_inner) => ::std::option::Option::Some(_inner),
            Self::DatabaseUnavailableException(_inner) => ::std::option::Option::Some(_inner),
            Self::ForbiddenException(_inner) => ::std::option::Option::Some(_inner),
            Self::HttpEndpointNotEnabledException(_inner) => ::std::option::Option::Some(_inner),
            Self::InternalServerErrorException(_inner) => ::std::option::Option::Some(_inner),
            Self::InvalidResourceStateException(_inner) => ::std::option::Option::Some(_inner),
            Self::InvalidSecretException(_inner) => ::std::option::Option::Some(_inner),
            Self::SecretsErrorException(_inner) => ::std::option::Option::Some(_inner),
            Self::ServiceUnavailableError(_inner) => ::std::option::Option::Some(_inner),
            Self::StatementTimeoutException(_inner) => ::std::option::Option::Some(_inner),
            Self::TransactionNotFoundException(_inner) => ::std::option::Option::Some(_inner),
            Self::UnsupportedResultException(_inner) => ::std::option::Option::Some(_inner),
            Self::Unhandled(_inner) => ::std::option::Option::Some(&*_inner.source),
        }
    }
}
impl ::std::fmt::Display for ExecuteStatementError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::AccessDeniedException(_inner) => _inner.fmt(f),
            Self::BadRequestException(_inner) => _inner.fmt(f),
            Self::DatabaseErrorException(_inner) => _inner.fmt(f),
            Self::DatabaseNotFoundException(_inner) => _inner.fmt(f),
            Self::DatabaseResumingException(_inner) => _inner.fmt(f),
            Self::DatabaseUnavailableException(_inner) => _inner.fmt(f),
            Self::ForbiddenException(_inner) => _inner.fmt(f),
            Self::HttpEndpointNotEnabledException(_inner) => _inner.fmt(f),
            Self::InternalServerErrorException(_inner) => _inner.fmt(f),
            Self::InvalidResourceStateException(_inner) => _inner.fmt(f),
            Self::InvalidSecretException(_inner) => _inner.fmt(f),
            Self::SecretsErrorException(_inner) => _inner.fmt(f),
            Self::ServiceUnavailableError(_inner) => _inner.fmt(f),
            Self::StatementTimeoutException(_inner) => _inner.fmt(f),
            Self::TransactionNotFoundException(_inner) => _inner.fmt(f),
            Self::UnsupportedResultException(_inner) => _inner.fmt(f),
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
impl ::aws_smithy_types::retry::ProvideErrorKind for ExecuteStatementError {
    fn code(&self) -> ::std::option::Option<&str> {
        ::aws_smithy_types::error::metadata::ProvideErrorMetadata::code(self)
    }
    fn retryable_error_kind(&self) -> ::std::option::Option<::aws_smithy_types::retry::ErrorKind> {
        ::std::option::Option::None
    }
}
impl ::aws_smithy_types::error::metadata::ProvideErrorMetadata for ExecuteStatementError {
    fn meta(&self) -> &::aws_smithy_types::error::ErrorMetadata {
        match self {
            Self::AccessDeniedException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::BadRequestException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::DatabaseErrorException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::DatabaseNotFoundException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::DatabaseResumingException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::DatabaseUnavailableException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::ForbiddenException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::HttpEndpointNotEnabledException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::InternalServerErrorException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::InvalidResourceStateException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::InvalidSecretException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::SecretsErrorException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::ServiceUnavailableError(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::StatementTimeoutException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::TransactionNotFoundException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::UnsupportedResultException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::Unhandled(_inner) => &_inner.meta,
        }
    }
}
impl ::aws_smithy_runtime_api::client::result::CreateUnhandledError for ExecuteStatementError {
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
impl ::aws_types::request_id::RequestId for crate::operation::execute_statement::ExecuteStatementError {
    fn request_id(&self) -> Option<&str> {
        self.meta().request_id()
    }
}

pub use crate::operation::execute_statement::_execute_statement_output::ExecuteStatementOutput;

pub use crate::operation::execute_statement::_execute_statement_input::ExecuteStatementInput;

mod _execute_statement_input;

mod _execute_statement_output;

/// Builders
pub mod builders;

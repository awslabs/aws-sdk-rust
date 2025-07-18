// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
/// Orchestration and serialization glue logic for `GetSnapshotBlock`.
#[derive(::std::clone::Clone, ::std::default::Default, ::std::fmt::Debug)]
#[non_exhaustive]
pub struct GetSnapshotBlock;
impl GetSnapshotBlock {
    /// Creates a new `GetSnapshotBlock`
    pub fn new() -> Self {
        Self
    }
    pub(crate) async fn orchestrate(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::get_snapshot_block::GetSnapshotBlockInput,
    ) -> ::std::result::Result<
        crate::operation::get_snapshot_block::GetSnapshotBlockOutput,
        ::aws_smithy_runtime_api::client::result::SdkError<
            crate::operation::get_snapshot_block::GetSnapshotBlockError,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >,
    > {
        let map_err = |err: ::aws_smithy_runtime_api::client::result::SdkError<
            ::aws_smithy_runtime_api::client::interceptors::context::Error,
            ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
        >| {
            err.map_service_error(|err| {
                err.downcast::<crate::operation::get_snapshot_block::GetSnapshotBlockError>()
                    .expect("correct error type")
            })
        };
        let context = Self::orchestrate_with_stop_point(runtime_plugins, input, ::aws_smithy_runtime::client::orchestrator::StopPoint::None)
            .await
            .map_err(map_err)?;
        let output = context.finalize().map_err(map_err)?;
        ::std::result::Result::Ok(
            output
                .downcast::<crate::operation::get_snapshot_block::GetSnapshotBlockOutput>()
                .expect("correct output type"),
        )
    }

    pub(crate) async fn orchestrate_with_stop_point(
        runtime_plugins: &::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins,
        input: crate::operation::get_snapshot_block::GetSnapshotBlockInput,
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
        ::aws_smithy_runtime::client::orchestrator::invoke_with_stop_point("EBS", "GetSnapshotBlock", input, runtime_plugins, stop_point)
            // Create a parent span for the entire operation. Includes a random, internal-only,
            // seven-digit ID for the operation orchestration so that it can be correlated in the logs.
            .instrument(::tracing::debug_span!(
                "EBS.GetSnapshotBlock",
                "rpc.service" = "EBS",
                "rpc.method" = "GetSnapshotBlock",
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
impl ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin for GetSnapshotBlock {
    fn config(&self) -> ::std::option::Option<::aws_smithy_types::config_bag::FrozenLayer> {
        let mut cfg = ::aws_smithy_types::config_bag::Layer::new("GetSnapshotBlock");

        cfg.store_put(::aws_smithy_runtime_api::client::ser_de::SharedRequestSerializer::new(
            GetSnapshotBlockRequestSerializer,
        ));
        cfg.store_put(::aws_smithy_runtime_api::client::ser_de::SharedResponseDeserializer::new(
            GetSnapshotBlockResponseDeserializer,
        ));

        cfg.store_put(::aws_smithy_runtime_api::client::auth::AuthSchemeOptionResolverParams::new(
            crate::config::auth::Params::builder()
                .operation_name("GetSnapshotBlock")
                .build()
                .expect("required fields set"),
        ));

        cfg.store_put(::aws_smithy_runtime_api::client::orchestrator::SensitiveOutput);
        cfg.store_put(::aws_smithy_runtime_api::client::orchestrator::Metadata::new("GetSnapshotBlock", "EBS"));
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
        let mut rcb = ::aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder::new("GetSnapshotBlock")
            .with_interceptor(::aws_smithy_runtime::client::stalled_stream_protection::StalledStreamProtectionInterceptor::default())
            .with_interceptor(GetSnapshotBlockEndpointParamsInterceptor)
            .with_retry_classifier(::aws_smithy_runtime::client::retries::classifiers::TransientErrorClassifier::<
                crate::operation::get_snapshot_block::GetSnapshotBlockError,
            >::new())
            .with_retry_classifier(::aws_smithy_runtime::client::retries::classifiers::ModeledAsRetryableClassifier::<
                crate::operation::get_snapshot_block::GetSnapshotBlockError,
            >::new())
            .with_retry_classifier(::aws_runtime::retries::classifiers::AwsErrorCodeClassifier::<
                crate::operation::get_snapshot_block::GetSnapshotBlockError,
            >::new());

        ::std::borrow::Cow::Owned(rcb)
    }
}

#[derive(Debug)]
struct GetSnapshotBlockResponseDeserializer;
impl ::aws_smithy_runtime_api::client::ser_de::DeserializeResponse for GetSnapshotBlockResponseDeserializer {
    fn deserialize_streaming(
        &self,
        response: &mut ::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
    ) -> ::std::option::Option<::aws_smithy_runtime_api::client::interceptors::context::OutputOrError> {
        #[allow(unused_mut)]
        let mut force_error = false;
        ::tracing::debug!(request_id = ?::aws_types::request_id::RequestId::request_id(response));

        // If this is an error, defer to the non-streaming parser
        if (!response.status().is_success() && response.status().as_u16() != 200) || force_error {
            return ::std::option::Option::None;
        }
        ::std::option::Option::Some(crate::protocol_serde::type_erase_result(
            crate::protocol_serde::shape_get_snapshot_block::de_get_snapshot_block_http_response(response),
        ))
    }

    fn deserialize_nonstreaming(
        &self,
        response: &::aws_smithy_runtime_api::client::orchestrator::HttpResponse,
    ) -> ::aws_smithy_runtime_api::client::interceptors::context::OutputOrError {
        // For streaming operations, we only hit this case if its an error
        let body = response.body().bytes().expect("body loaded");
        crate::protocol_serde::type_erase_result(crate::protocol_serde::shape_get_snapshot_block::de_get_snapshot_block_http_error(
            response.status().as_u16(),
            response.headers(),
            body,
        ))
    }
}
#[derive(Debug)]
struct GetSnapshotBlockRequestSerializer;
impl ::aws_smithy_runtime_api::client::ser_de::SerializeRequest for GetSnapshotBlockRequestSerializer {
    #[allow(unused_mut, clippy::let_and_return, clippy::needless_borrow, clippy::useless_conversion)]
    fn serialize_input(
        &self,
        input: ::aws_smithy_runtime_api::client::interceptors::context::Input,
        _cfg: &mut ::aws_smithy_types::config_bag::ConfigBag,
    ) -> ::std::result::Result<::aws_smithy_runtime_api::client::orchestrator::HttpRequest, ::aws_smithy_runtime_api::box_error::BoxError> {
        let input = input
            .downcast::<crate::operation::get_snapshot_block::GetSnapshotBlockInput>()
            .expect("correct type");
        let _header_serialization_settings = _cfg
            .load::<crate::serialization_settings::HeaderSerializationSettings>()
            .cloned()
            .unwrap_or_default();
        let mut request_builder = {
            fn uri_base(
                _input: &crate::operation::get_snapshot_block::GetSnapshotBlockInput,
                output: &mut ::std::string::String,
            ) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::BuildError> {
                use ::std::fmt::Write as _;
                let input_1 = &_input.snapshot_id;
                let input_1 = input_1
                    .as_ref()
                    .ok_or_else(|| ::aws_smithy_types::error::operation::BuildError::missing_field("snapshot_id", "cannot be empty or unset"))?;
                let snapshot_id = ::aws_smithy_http::label::fmt_string(input_1, ::aws_smithy_http::label::EncodingStrategy::Default);
                if snapshot_id.is_empty() {
                    return ::std::result::Result::Err(::aws_smithy_types::error::operation::BuildError::missing_field(
                        "snapshot_id",
                        "cannot be empty or unset",
                    ));
                }
                let input_2 = &_input.block_index;
                let input_2 = input_2
                    .as_ref()
                    .ok_or_else(|| ::aws_smithy_types::error::operation::BuildError::missing_field("block_index", "cannot be empty or unset"))?;
                let mut block_index_encoder = ::aws_smithy_types::primitive::Encoder::from(*input_2);
                let block_index = block_index_encoder.encode();
                if block_index.is_empty() {
                    return ::std::result::Result::Err(::aws_smithy_types::error::operation::BuildError::missing_field(
                        "block_index",
                        "cannot be empty or unset",
                    ));
                }
                ::std::write!(
                    output,
                    "/snapshots/{SnapshotId}/blocks/{BlockIndex}",
                    SnapshotId = snapshot_id,
                    BlockIndex = block_index
                )
                .expect("formatting should succeed");
                ::std::result::Result::Ok(())
            }
            fn uri_query(
                _input: &crate::operation::get_snapshot_block::GetSnapshotBlockInput,
                mut output: &mut ::std::string::String,
            ) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::BuildError> {
                let mut query = ::aws_smithy_http::query::Writer::new(output);
                let inner_3 = &_input.block_token;
                let inner_3 = inner_3
                    .as_ref()
                    .ok_or_else(|| ::aws_smithy_types::error::operation::BuildError::missing_field("block_token", "cannot be empty or unset"))?;
                if inner_3.is_empty() {
                    return ::std::result::Result::Err(::aws_smithy_types::error::operation::BuildError::missing_field(
                        "block_token",
                        "cannot be empty or unset",
                    ));
                }
                query.push_kv("blockToken", &::aws_smithy_http::query::fmt_string(inner_3));
                ::std::result::Result::Ok(())
            }
            #[allow(clippy::unnecessary_wraps)]
            fn update_http_builder(
                input: &crate::operation::get_snapshot_block::GetSnapshotBlockInput,
                builder: ::http::request::Builder,
            ) -> ::std::result::Result<::http::request::Builder, ::aws_smithy_types::error::operation::BuildError> {
                let mut uri = ::std::string::String::new();
                uri_base(input, &mut uri)?;
                uri_query(input, &mut uri)?;
                ::std::result::Result::Ok(builder.method("GET").uri(uri))
            }
            let mut builder = update_http_builder(&input, ::http::request::Builder::new())?;
            builder
        };
        let body = ::aws_smithy_types::body::SdkBody::from("");

        ::std::result::Result::Ok(request_builder.body(body).expect("valid request").try_into().unwrap())
    }
}
#[derive(Debug)]
struct GetSnapshotBlockEndpointParamsInterceptor;

impl ::aws_smithy_runtime_api::client::interceptors::Intercept for GetSnapshotBlockEndpointParamsInterceptor {
    fn name(&self) -> &'static str {
        "GetSnapshotBlockEndpointParamsInterceptor"
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
            .downcast_ref::<GetSnapshotBlockInput>()
            .ok_or("failed to downcast to GetSnapshotBlockInput")?;

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

#[allow(unreachable_code, unused_variables)]
#[cfg(test)]
mod get_snapshot_block_test {

    /// This test case validates case insensitive parsing of `message`
    /// Test ID: LowercaseMessage
    #[::tokio::test]
    #[::tracing_test::traced_test]
    async fn lowercase_message_response() {
        let expected_output = crate::types::error::ValidationException::builder()
            .set_message(::std::option::Option::Some("1 validation error detected".to_owned()))
            .build();
        let mut http_response = ::aws_smithy_runtime_api::http::Response::try_from(
            ::http::response::Builder::new()
                .header("content-length", "77")
                .header("content-type", "application/json")
                .header("date", "Wed, 30 Jun 2021 23:42:27 GMT")
                .header(
                    "x-amzn-errortype",
                    "ValidationException:http://internal.amazon.com/coral/com.amazon.coral.validate/",
                )
                .header("x-amzn-requestid", "2af8f013-250a-4f6e-88ae-6dd7f6e12807")
                .status(400)
                .body(::aws_smithy_types::body::SdkBody::from(
                    "{\n  \"message\": \"1 validation error detected\"\n}\n",
                ))
                .unwrap(),
        )
        .unwrap();
        use ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin;
        use ::aws_smithy_runtime_api::client::ser_de::DeserializeResponse;

        let op = crate::operation::get_snapshot_block::GetSnapshotBlock::new();
        let config = op.config().expect("the operation has config");
        let de = config
            .load::<::aws_smithy_runtime_api::client::ser_de::SharedResponseDeserializer>()
            .expect("the config must have a deserializer");

        let parsed = de.deserialize_streaming(&mut http_response);
        let parsed = parsed.unwrap_or_else(|| {
            let http_response = http_response.map(|body| {
                ::aws_smithy_types::body::SdkBody::from(::bytes::Bytes::copy_from_slice(&::aws_smithy_protocol_test::decode_body_data(
                    body.bytes().unwrap(),
                    ::aws_smithy_protocol_test::MediaType::from("application/json"),
                )))
            });
            de.deserialize_nonstreaming(&http_response)
        });
        let parsed = parsed.expect_err("should be error response");
        let parsed: &crate::operation::get_snapshot_block::GetSnapshotBlockError =
            parsed.as_operation_error().expect("operation error").downcast_ref().unwrap();
        if let crate::operation::get_snapshot_block::GetSnapshotBlockError::ValidationException(parsed) = parsed {
            ::pretty_assertions::assert_eq!(parsed.message, expected_output.message, "Unexpected value for `message`");
            ::pretty_assertions::assert_eq!(parsed.reason, expected_output.reason, "Unexpected value for `reason`");
        } else {
            panic!("wrong variant: Got: {:?}. Expected: {:?}", parsed, expected_output);
        }
    }

    /// This test case validates case insensitive parsing of `message`
    /// Test ID: UppercaseMessage
    #[::tokio::test]
    #[::tracing_test::traced_test]
    async fn uppercase_message_response() {
        let expected_output = crate::types::error::ValidationException::builder()
            .set_message(::std::option::Option::Some("Invalid volume size: 99999999999".to_owned()))
            .set_reason(::std::option::Option::Some(
                "INVALID_VOLUME_SIZE"
                    .parse::<crate::types::ValidationExceptionReason>()
                    .expect("static value validated to member"),
            ))
            .build();
        let mut http_response = ::aws_smithy_runtime_api::http::Response::try_from(
            ::http::response::Builder::new()
                .header("content-length", "77")
                .header("content-type", "application/json")
                .header("date", "Wed, 30 Jun 2021 23:42:27 GMT")
                .header(
                    "x-amzn-errortype",
                    "ValidationException:http://internal.amazon.com/coral/com.amazon.zeppelindataservice/",
                )
                .header("x-amzn-requestid", "2af8f013-250a-4f6e-88ae-6dd7f6e12807")
                .status(400)
                .body(::aws_smithy_types::body::SdkBody::from(
                    "{\"Message\":\"Invalid volume size: 99999999999\",\"Reason\":\"INVALID_VOLUME_SIZE\"}\n",
                ))
                .unwrap(),
        )
        .unwrap();
        use ::aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin;
        use ::aws_smithy_runtime_api::client::ser_de::DeserializeResponse;

        let op = crate::operation::get_snapshot_block::GetSnapshotBlock::new();
        let config = op.config().expect("the operation has config");
        let de = config
            .load::<::aws_smithy_runtime_api::client::ser_de::SharedResponseDeserializer>()
            .expect("the config must have a deserializer");

        let parsed = de.deserialize_streaming(&mut http_response);
        let parsed = parsed.unwrap_or_else(|| {
            let http_response = http_response.map(|body| {
                ::aws_smithy_types::body::SdkBody::from(::bytes::Bytes::copy_from_slice(&::aws_smithy_protocol_test::decode_body_data(
                    body.bytes().unwrap(),
                    ::aws_smithy_protocol_test::MediaType::from("application/json"),
                )))
            });
            de.deserialize_nonstreaming(&http_response)
        });
        let parsed = parsed.expect_err("should be error response");
        let parsed: &crate::operation::get_snapshot_block::GetSnapshotBlockError =
            parsed.as_operation_error().expect("operation error").downcast_ref().unwrap();
        if let crate::operation::get_snapshot_block::GetSnapshotBlockError::ValidationException(parsed) = parsed {
            ::pretty_assertions::assert_eq!(parsed.message, expected_output.message, "Unexpected value for `message`");
            ::pretty_assertions::assert_eq!(parsed.reason, expected_output.reason, "Unexpected value for `reason`");
        } else {
            panic!("wrong variant: Got: {:?}. Expected: {:?}", parsed, expected_output);
        }
    }
}

/// Error type for the `GetSnapshotBlockError` operation.
#[non_exhaustive]
#[derive(::std::fmt::Debug)]
pub enum GetSnapshotBlockError {
    /// <p>You do not have sufficient access to perform this action.</p>
    AccessDeniedException(crate::types::error::AccessDeniedException),
    /// <p>An internal error has occurred. For more information see <a href="https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/error-retries.html">Error retries</a>.</p>
    InternalServerException(crate::types::error::InternalServerException),
    /// <p>The number of API requests has exceeded the maximum allowed API request throttling limit for the snapshot. For more information see <a href="https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/error-retries.html">Error retries</a>.</p>
    RequestThrottledException(crate::types::error::RequestThrottledException),
    /// <p>The specified resource does not exist.</p>
    ResourceNotFoundException(crate::types::error::ResourceNotFoundException),
    /// <p>Your current service quotas do not allow you to perform this action.</p>
    ServiceQuotaExceededException(crate::types::error::ServiceQuotaExceededException),
    /// <p>The input fails to satisfy the constraints of the EBS direct APIs.</p>
    ValidationException(crate::types::error::ValidationException),
    /// An unexpected error occurred (e.g., invalid JSON returned by the service or an unknown error code).
    #[deprecated(note = "Matching `Unhandled` directly is not forwards compatible. Instead, match using a \
    variable wildcard pattern and check `.code()`:
     \
    &nbsp;&nbsp;&nbsp;`err if err.code() == Some(\"SpecificExceptionCode\") => { /* handle the error */ }`
     \
    See [`ProvideErrorMetadata`](#impl-ProvideErrorMetadata-for-GetSnapshotBlockError) for what information is available for the error.")]
    Unhandled(crate::error::sealed_unhandled::Unhandled),
}
impl GetSnapshotBlockError {
    /// Creates the `GetSnapshotBlockError::Unhandled` variant from any error type.
    pub fn unhandled(
        err: impl ::std::convert::Into<::std::boxed::Box<dyn ::std::error::Error + ::std::marker::Send + ::std::marker::Sync + 'static>>,
    ) -> Self {
        Self::Unhandled(crate::error::sealed_unhandled::Unhandled {
            source: err.into(),
            meta: ::std::default::Default::default(),
        })
    }

    /// Creates the `GetSnapshotBlockError::Unhandled` variant from an [`ErrorMetadata`](::aws_smithy_types::error::ErrorMetadata).
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
            Self::InternalServerException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::RequestThrottledException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::ResourceNotFoundException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::ServiceQuotaExceededException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::ValidationException(e) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(e),
            Self::Unhandled(e) => &e.meta,
        }
    }
    /// Returns `true` if the error kind is `GetSnapshotBlockError::AccessDeniedException`.
    pub fn is_access_denied_exception(&self) -> bool {
        matches!(self, Self::AccessDeniedException(_))
    }
    /// Returns `true` if the error kind is `GetSnapshotBlockError::InternalServerException`.
    pub fn is_internal_server_exception(&self) -> bool {
        matches!(self, Self::InternalServerException(_))
    }
    /// Returns `true` if the error kind is `GetSnapshotBlockError::RequestThrottledException`.
    pub fn is_request_throttled_exception(&self) -> bool {
        matches!(self, Self::RequestThrottledException(_))
    }
    /// Returns `true` if the error kind is `GetSnapshotBlockError::ResourceNotFoundException`.
    pub fn is_resource_not_found_exception(&self) -> bool {
        matches!(self, Self::ResourceNotFoundException(_))
    }
    /// Returns `true` if the error kind is `GetSnapshotBlockError::ServiceQuotaExceededException`.
    pub fn is_service_quota_exceeded_exception(&self) -> bool {
        matches!(self, Self::ServiceQuotaExceededException(_))
    }
    /// Returns `true` if the error kind is `GetSnapshotBlockError::ValidationException`.
    pub fn is_validation_exception(&self) -> bool {
        matches!(self, Self::ValidationException(_))
    }
}
impl ::std::error::Error for GetSnapshotBlockError {
    fn source(&self) -> ::std::option::Option<&(dyn ::std::error::Error + 'static)> {
        match self {
            Self::AccessDeniedException(_inner) => ::std::option::Option::Some(_inner),
            Self::InternalServerException(_inner) => ::std::option::Option::Some(_inner),
            Self::RequestThrottledException(_inner) => ::std::option::Option::Some(_inner),
            Self::ResourceNotFoundException(_inner) => ::std::option::Option::Some(_inner),
            Self::ServiceQuotaExceededException(_inner) => ::std::option::Option::Some(_inner),
            Self::ValidationException(_inner) => ::std::option::Option::Some(_inner),
            Self::Unhandled(_inner) => ::std::option::Option::Some(&*_inner.source),
        }
    }
}
impl ::std::fmt::Display for GetSnapshotBlockError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::AccessDeniedException(_inner) => _inner.fmt(f),
            Self::InternalServerException(_inner) => _inner.fmt(f),
            Self::RequestThrottledException(_inner) => _inner.fmt(f),
            Self::ResourceNotFoundException(_inner) => _inner.fmt(f),
            Self::ServiceQuotaExceededException(_inner) => _inner.fmt(f),
            Self::ValidationException(_inner) => _inner.fmt(f),
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
impl ::aws_smithy_types::retry::ProvideErrorKind for GetSnapshotBlockError {
    fn code(&self) -> ::std::option::Option<&str> {
        ::aws_smithy_types::error::metadata::ProvideErrorMetadata::code(self)
    }
    fn retryable_error_kind(&self) -> ::std::option::Option<::aws_smithy_types::retry::ErrorKind> {
        ::std::option::Option::None
    }
}
impl ::aws_smithy_types::error::metadata::ProvideErrorMetadata for GetSnapshotBlockError {
    fn meta(&self) -> &::aws_smithy_types::error::ErrorMetadata {
        match self {
            Self::AccessDeniedException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::InternalServerException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::RequestThrottledException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::ResourceNotFoundException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::ServiceQuotaExceededException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::ValidationException(_inner) => ::aws_smithy_types::error::metadata::ProvideErrorMetadata::meta(_inner),
            Self::Unhandled(_inner) => &_inner.meta,
        }
    }
}
impl ::aws_smithy_runtime_api::client::result::CreateUnhandledError for GetSnapshotBlockError {
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
impl ::aws_types::request_id::RequestId for crate::operation::get_snapshot_block::GetSnapshotBlockError {
    fn request_id(&self) -> Option<&str> {
        self.meta().request_id()
    }
}

pub use crate::operation::get_snapshot_block::_get_snapshot_block_output::GetSnapshotBlockOutput;

pub use crate::operation::get_snapshot_block::_get_snapshot_block_input::GetSnapshotBlockInput;

mod _get_snapshot_block_input;

mod _get_snapshot_block_output;

/// Builders
pub mod builders;

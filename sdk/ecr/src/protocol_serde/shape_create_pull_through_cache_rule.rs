// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_create_pull_through_cache_rule_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleOutput,
    crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "InvalidParameterException" => {
            crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::InvalidParameterException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidParameterExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_invalid_parameter_exception::de_invalid_parameter_exception_json_err(_response_body, output)
                            .map_err(crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "LimitExceededException" => crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::LimitExceededException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::LimitExceededExceptionBuilder::default();
                output = crate::protocol_serde::shape_limit_exceeded_exception::de_limit_exceeded_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "PullThroughCacheRuleAlreadyExistsException" => {
            crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::PullThroughCacheRuleAlreadyExistsException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::PullThroughCacheRuleAlreadyExistsExceptionBuilder::default();
                    output = crate::protocol_serde::shape_pull_through_cache_rule_already_exists_exception::de_pull_through_cache_rule_already_exists_exception_json_err(_response_body, output).map_err(crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "SecretNotFoundException" => crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::SecretNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::SecretNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_secret_not_found_exception::de_secret_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ServerException" => crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::ServerException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ServerExceptionBuilder::default();
                output = crate::protocol_serde::shape_server_exception::de_server_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "UnableToAccessSecretException" => {
            crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::UnableToAccessSecretException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::UnableToAccessSecretExceptionBuilder::default();
                    output = crate::protocol_serde::shape_unable_to_access_secret_exception::de_unable_to_access_secret_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "UnableToDecryptSecretValueException" => {
            crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::UnableToDecryptSecretValueException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::UnableToDecryptSecretValueExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_unable_to_decrypt_secret_value_exception::de_unable_to_decrypt_secret_value_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "UnsupportedUpstreamRegistryException" => {
            crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::UnsupportedUpstreamRegistryException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::UnsupportedUpstreamRegistryExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_unsupported_upstream_registry_exception::de_unsupported_upstream_registry_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "ValidationException" => crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::ValidationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationExceptionBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_create_pull_through_cache_rule_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleOutput,
    crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::create_pull_through_cache_rule::builders::CreatePullThroughCacheRuleOutputBuilder::default();
        output = crate::protocol_serde::shape_create_pull_through_cache_rule::de_create_pull_through_cache_rule(_response_body, output)
            .map_err(crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_create_pull_through_cache_rule_input(
    input: &crate::operation::create_pull_through_cache_rule::CreatePullThroughCacheRuleInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_create_pull_through_cache_rule_input::ser_create_pull_through_cache_rule_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_create_pull_through_cache_rule(
    value: &[u8],
    mut builder: crate::operation::create_pull_through_cache_rule::builders::CreatePullThroughCacheRuleOutputBuilder,
) -> ::std::result::Result<
    crate::operation::create_pull_through_cache_rule::builders::CreatePullThroughCacheRuleOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "ecrRepositoryPrefix" => {
                    builder = builder.set_ecr_repository_prefix(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "upstreamRegistryUrl" => {
                    builder = builder.set_upstream_registry_url(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "createdAt" => {
                    builder = builder.set_created_at(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                        tokens.next(),
                        ::aws_smithy_types::date_time::Format::EpochSeconds,
                    )?);
                }
                "registryId" => {
                    builder = builder.set_registry_id(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "upstreamRegistry" => {
                    builder = builder.set_upstream_registry(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| crate::types::UpstreamRegistry::from(u.as_ref())))
                            .transpose()?,
                    );
                }
                "credentialArn" => {
                    builder = builder.set_credential_arn(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "customRoleArn" => {
                    builder = builder.set_custom_role_arn(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "upstreamRepositoryPrefix" => {
                    builder = builder.set_upstream_repository_prefix(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                _ => ::aws_smithy_json::deserialize::token::skip_value(tokens)?,
            },
            other => {
                return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(format!(
                    "expected object key or end object, found: {:?}",
                    other
                )))
            }
        }
    }
    if tokens.next().is_some() {
        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "found more JSON tokens after completing parsing",
        ));
    }
    Ok(builder)
}

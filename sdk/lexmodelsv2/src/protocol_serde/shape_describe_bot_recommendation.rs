// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_bot_recommendation_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::describe_bot_recommendation::DescribeBotRecommendationOutput,
    crate::operation::describe_bot_recommendation::DescribeBotRecommendationError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => {
            return Err(crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::unhandled(
                generic,
            ))
        }
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "InternalServerException" => crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::InternalServerException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_server_exception::de_internal_server_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceNotFoundException" => crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::ResourceNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ThrottlingException" => crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::ThrottlingException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingExceptionBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::unhandled)?;
                output = output.set_retry_after_seconds(
                    crate::protocol_serde::shape_throttling_exception::de_retry_after_seconds_header(_response_headers).map_err(|_| {
                        crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::unhandled(
                            "Failed to parse retryAfterSeconds from header `Retry-After",
                        )
                    })?,
                );
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ValidationException" => crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::ValidationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationExceptionBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_bot_recommendation_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::describe_bot_recommendation::DescribeBotRecommendationOutput,
    crate::operation::describe_bot_recommendation::DescribeBotRecommendationError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::describe_bot_recommendation::builders::DescribeBotRecommendationOutputBuilder::default();
        output = crate::protocol_serde::shape_describe_bot_recommendation::de_describe_bot_recommendation(_response_body, output)
            .map_err(crate::operation::describe_bot_recommendation::DescribeBotRecommendationError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub(crate) fn de_describe_bot_recommendation(
    value: &[u8],
    mut builder: crate::operation::describe_bot_recommendation::builders::DescribeBotRecommendationOutputBuilder,
) -> ::std::result::Result<
    crate::operation::describe_bot_recommendation::builders::DescribeBotRecommendationOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "botId" => {
                    builder = builder.set_bot_id(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "botRecommendationId" => {
                    builder = builder.set_bot_recommendation_id(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "botRecommendationResults" => {
                    builder = builder.set_bot_recommendation_results(
                        crate::protocol_serde::shape_bot_recommendation_results::de_bot_recommendation_results(tokens)?,
                    );
                }
                "botRecommendationStatus" => {
                    builder = builder.set_bot_recommendation_status(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| crate::types::BotRecommendationStatus::from(u.as_ref())))
                            .transpose()?,
                    );
                }
                "botVersion" => {
                    builder = builder.set_bot_version(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "creationDateTime" => {
                    builder = builder.set_creation_date_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                        tokens.next(),
                        ::aws_smithy_types::date_time::Format::EpochSeconds,
                    )?);
                }
                "encryptionSetting" => {
                    builder = builder.set_encryption_setting(crate::protocol_serde::shape_encryption_setting::de_encryption_setting(tokens)?);
                }
                "failureReasons" => {
                    builder = builder.set_failure_reasons(crate::protocol_serde::shape_failure_reasons::de_failure_reasons(tokens)?);
                }
                "lastUpdatedDateTime" => {
                    builder = builder.set_last_updated_date_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                        tokens.next(),
                        ::aws_smithy_types::date_time::Format::EpochSeconds,
                    )?);
                }
                "localeId" => {
                    builder = builder.set_locale_id(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "transcriptSourceSetting" => {
                    builder = builder.set_transcript_source_setting(
                        crate::protocol_serde::shape_transcript_source_setting::de_transcript_source_setting(tokens)?,
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

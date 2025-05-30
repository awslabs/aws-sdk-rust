// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_create_presigned_url_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::create_presigned_url::CreatePresignedUrlOutput,
    crate::operation::create_presigned_url::CreatePresignedUrlError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AccessDeniedException" => crate::operation::create_presigned_url::CreatePresignedUrlError::AccessDeniedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AccessDeniedExceptionBuilder::default();
                output = crate::protocol_serde::shape_access_denied_exception::de_access_denied_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::access_denied_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?
            };
            tmp
        }),
        "InternalServerException" => crate::operation::create_presigned_url::CreatePresignedUrlError::InternalServerException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_server_exception::de_internal_server_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?;
                output = output.set_retry_after_seconds(
                    crate::protocol_serde::shape_internal_server_exception::de_retry_after_seconds_header(_response_headers).map_err(|_| {
                        crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled(
                            "Failed to parse retryAfterSeconds from header `Retry-After",
                        )
                    })?,
                );
                let output = output.meta(generic);
                crate::serde_util::internal_server_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?
            };
            tmp
        }),
        "ThrottlingException" => crate::operation::create_presigned_url::CreatePresignedUrlError::ThrottlingException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingExceptionBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?;
                output = output.set_retry_after_seconds(
                    crate::protocol_serde::shape_throttling_exception::de_retry_after_seconds_header(_response_headers).map_err(|_| {
                        crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled(
                            "Failed to parse retryAfterSeconds from header `Retry-After",
                        )
                    })?,
                );
                let output = output.meta(generic);
                crate::serde_util::throttling_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?
            };
            tmp
        }),
        "UnauthorizedException" => crate::operation::create_presigned_url::CreatePresignedUrlError::UnauthorizedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::UnauthorizedExceptionBuilder::default();
                output = crate::protocol_serde::shape_unauthorized_exception::de_unauthorized_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::unauthorized_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?
            };
            tmp
        }),
        "ValidationException" => crate::operation::create_presigned_url::CreatePresignedUrlError::ValidationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationExceptionBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::validation_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?
            };
            tmp
        }),
        _ => crate::operation::create_presigned_url::CreatePresignedUrlError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_create_presigned_url_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::create_presigned_url::CreatePresignedUrlOutput,
    crate::operation::create_presigned_url::CreatePresignedUrlError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::create_presigned_url::builders::CreatePresignedUrlOutputBuilder::default();
        output = crate::protocol_serde::shape_create_presigned_url::de_create_presigned_url(_response_body, output)
            .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::create_presigned_url_output_output_correct_errors(output)
            .build()
            .map_err(crate::operation::create_presigned_url::CreatePresignedUrlError::unhandled)?
    })
}

pub fn ser_create_presigned_url_headers(
    input: &crate::operation::create_presigned_url::CreatePresignedUrlInput,
    mut builder: ::http::request::Builder,
) -> std::result::Result<::http::request::Builder, ::aws_smithy_types::error::operation::BuildError> {
    if let ::std::option::Option::Some(inner_1) = &input.instance_id {
        let formatted_2 = inner_1.as_str();
        let header_value = formatted_2;
        let header_value: ::http::HeaderValue = header_value.parse().map_err(|err| {
            ::aws_smithy_types::error::operation::BuildError::invalid_field(
                "instance_id",
                format!("`{}` cannot be used as a header value: {}", &header_value, err),
            )
        })?;
        builder = builder.header("instance-id", header_value);
    }
    Ok(builder)
}

pub fn ser_create_presigned_url_input(
    input: &crate::operation::create_presigned_url::CreatePresignedUrlInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_create_presigned_url_input::ser_create_presigned_url_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_create_presigned_url(
    value: &[u8],
    mut builder: crate::operation::create_presigned_url::builders::CreatePresignedUrlOutputBuilder,
) -> ::std::result::Result<
    crate::operation::create_presigned_url::builders::CreatePresignedUrlOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "fileId" => {
                    builder = builder.set_file_id(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "presignedUrl" => {
                    builder = builder.set_presigned_url(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "presignedUrlExpiration" => {
                    builder = builder.set_presigned_url_expiration(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                        tokens.next(),
                        ::aws_smithy_types::date_time::Format::DateTimeWithOffset,
                    )?);
                }
                "presignedUrlFields" => {
                    builder = builder.set_presigned_url_fields(crate::protocol_serde::shape_presigned_url_fields::de_presigned_url_fields(tokens)?);
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

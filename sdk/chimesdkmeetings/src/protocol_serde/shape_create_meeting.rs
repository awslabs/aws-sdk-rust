// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_create_meeting_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::create_meeting::CreateMeetingOutput, crate::operation::create_meeting::CreateMeetingError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::create_meeting::CreateMeetingError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::create_meeting::CreateMeetingError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "BadRequestException" => crate::operation::create_meeting::CreateMeetingError::BadRequestException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::BadRequestExceptionBuilder::default();
                output = crate::protocol_serde::shape_bad_request_exception::de_bad_request_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_meeting::CreateMeetingError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ConflictException" => crate::operation::create_meeting::CreateMeetingError::ConflictException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ConflictExceptionBuilder::default();
                output = crate::protocol_serde::shape_conflict_exception::de_conflict_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_meeting::CreateMeetingError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ForbiddenException" => crate::operation::create_meeting::CreateMeetingError::ForbiddenException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ForbiddenExceptionBuilder::default();
                output = crate::protocol_serde::shape_forbidden_exception::de_forbidden_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_meeting::CreateMeetingError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "LimitExceededException" => crate::operation::create_meeting::CreateMeetingError::LimitExceededException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::LimitExceededExceptionBuilder::default();
                output = crate::protocol_serde::shape_limit_exceeded_exception::de_limit_exceeded_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_meeting::CreateMeetingError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ServiceFailureException" => crate::operation::create_meeting::CreateMeetingError::ServiceFailureException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ServiceFailureExceptionBuilder::default();
                output = crate::protocol_serde::shape_service_failure_exception::de_service_failure_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_meeting::CreateMeetingError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ServiceUnavailableException" => crate::operation::create_meeting::CreateMeetingError::ServiceUnavailableException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ServiceUnavailableExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_service_unavailable_exception::de_service_unavailable_exception_json_err(_response_body, output)
                        .map_err(crate::operation::create_meeting::CreateMeetingError::unhandled)?;
                output = output.set_retry_after_seconds(
                    crate::protocol_serde::shape_service_unavailable_exception::de_retry_after_seconds_header(_response_headers).map_err(|_| {
                        crate::operation::create_meeting::CreateMeetingError::unhandled("Failed to parse RetryAfterSeconds from header `Retry-After")
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
        "ThrottlingException" => crate::operation::create_meeting::CreateMeetingError::ThrottlingException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingExceptionBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_meeting::CreateMeetingError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "UnauthorizedException" => crate::operation::create_meeting::CreateMeetingError::UnauthorizedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::UnauthorizedExceptionBuilder::default();
                output = crate::protocol_serde::shape_unauthorized_exception::de_unauthorized_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_meeting::CreateMeetingError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::create_meeting::CreateMeetingError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_create_meeting_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::create_meeting::CreateMeetingOutput, crate::operation::create_meeting::CreateMeetingError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::create_meeting::builders::CreateMeetingOutputBuilder::default();
        output = crate::protocol_serde::shape_create_meeting::de_create_meeting(_response_body, output)
            .map_err(crate::operation::create_meeting::CreateMeetingError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_create_meeting_input(
    input: &crate::operation::create_meeting::CreateMeetingInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_create_meeting_input::ser_create_meeting_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_create_meeting(
    value: &[u8],
    mut builder: crate::operation::create_meeting::builders::CreateMeetingOutputBuilder,
) -> ::std::result::Result<
    crate::operation::create_meeting::builders::CreateMeetingOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "Meeting" => {
                    builder = builder.set_meeting(crate::protocol_serde::shape_meeting::de_meeting(tokens)?);
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

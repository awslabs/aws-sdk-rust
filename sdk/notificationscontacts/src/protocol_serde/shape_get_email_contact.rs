// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_get_email_contact_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::get_email_contact::GetEmailContactOutput, crate::operation::get_email_contact::GetEmailContactError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::get_email_contact::GetEmailContactError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::get_email_contact::GetEmailContactError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AccessDeniedException" => crate::operation::get_email_contact::GetEmailContactError::AccessDeniedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AccessDeniedExceptionBuilder::default();
                output = crate::protocol_serde::shape_access_denied_exception::de_access_denied_exception_json_err(_response_body, output)
                    .map_err(crate::operation::get_email_contact::GetEmailContactError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::access_denied_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::get_email_contact::GetEmailContactError::unhandled)?
            };
            tmp
        }),
        "InternalServerException" => crate::operation::get_email_contact::GetEmailContactError::InternalServerException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_server_exception::de_internal_server_exception_json_err(_response_body, output)
                    .map_err(crate::operation::get_email_contact::GetEmailContactError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::internal_server_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::get_email_contact::GetEmailContactError::unhandled)?
            };
            tmp
        }),
        "ResourceNotFoundException" => crate::operation::get_email_contact::GetEmailContactError::ResourceNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::get_email_contact::GetEmailContactError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::resource_not_found_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::get_email_contact::GetEmailContactError::unhandled)?
            };
            tmp
        }),
        "ThrottlingException" => crate::operation::get_email_contact::GetEmailContactError::ThrottlingException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingExceptionBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(_response_body, output)
                    .map_err(crate::operation::get_email_contact::GetEmailContactError::unhandled)?;
                output = output.set_retry_after_seconds(
                    crate::protocol_serde::shape_throttling_exception::de_retry_after_seconds_header(_response_headers).map_err(|_| {
                        crate::operation::get_email_contact::GetEmailContactError::unhandled(
                            "Failed to parse retryAfterSeconds from header `Retry-After",
                        )
                    })?,
                );
                let output = output.meta(generic);
                crate::serde_util::throttling_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::get_email_contact::GetEmailContactError::unhandled)?
            };
            tmp
        }),
        "ValidationException" => crate::operation::get_email_contact::GetEmailContactError::ValidationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationExceptionBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_json_err(_response_body, output)
                    .map_err(crate::operation::get_email_contact::GetEmailContactError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::validation_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::get_email_contact::GetEmailContactError::unhandled)?
            };
            tmp
        }),
        _ => crate::operation::get_email_contact::GetEmailContactError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_get_email_contact_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::get_email_contact::GetEmailContactOutput, crate::operation::get_email_contact::GetEmailContactError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::get_email_contact::builders::GetEmailContactOutputBuilder::default();
        output = crate::protocol_serde::shape_get_email_contact::de_get_email_contact(_response_body, output)
            .map_err(crate::operation::get_email_contact::GetEmailContactError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::get_email_contact_output_output_correct_errors(output).build()
    })
}

pub(crate) fn de_get_email_contact(
    value: &[u8],
    mut builder: crate::operation::get_email_contact::builders::GetEmailContactOutputBuilder,
) -> ::std::result::Result<
    crate::operation::get_email_contact::builders::GetEmailContactOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "emailContact" => {
                    builder = builder.set_email_contact(crate::protocol_serde::shape_email_contact::de_email_contact(tokens)?);
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

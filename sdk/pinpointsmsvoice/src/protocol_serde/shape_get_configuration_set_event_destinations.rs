// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_get_configuration_set_event_destinations_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsOutput,
    crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => {
            return Err(crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError::unhandled(generic))
        }
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "BadRequestException" => {
            crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError::BadRequestException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::BadRequestExceptionBuilder::default();
                    output = crate::protocol_serde::shape_bad_request_exception::de_bad_request_exception_json_err(_response_body, output)
                        .map_err(crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InternalServiceErrorException" => {
            crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError::InternalServiceErrorException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InternalServiceErrorExceptionBuilder::default();
                    output = crate::protocol_serde::shape_internal_service_error_exception::de_internal_service_error_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "NotFoundException" => {
            crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError::NotFoundException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::NotFoundExceptionBuilder::default();
                    output = crate::protocol_serde::shape_not_found_exception::de_not_found_exception_json_err(_response_body, output)
                        .map_err(crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "TooManyRequestsException" => {
            crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError::TooManyRequestsException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::TooManyRequestsExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_too_many_requests_exception::de_too_many_requests_exception_json_err(_response_body, output)
                            .map_err(
                                crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError::unhandled,
                            )?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        _ => crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_get_configuration_set_event_destinations_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsOutput,
    crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output =
            crate::operation::get_configuration_set_event_destinations::builders::GetConfigurationSetEventDestinationsOutputBuilder::default();
        output = crate::protocol_serde::shape_get_configuration_set_event_destinations::de_get_configuration_set_event_destinations(
            _response_body,
            output,
        )
        .map_err(crate::operation::get_configuration_set_event_destinations::GetConfigurationSetEventDestinationsError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub(crate) fn de_get_configuration_set_event_destinations(
    value: &[u8],
    mut builder: crate::operation::get_configuration_set_event_destinations::builders::GetConfigurationSetEventDestinationsOutputBuilder,
) -> ::std::result::Result<
    crate::operation::get_configuration_set_event_destinations::builders::GetConfigurationSetEventDestinationsOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "EventDestinations" => {
                    builder = builder.set_event_destinations(crate::protocol_serde::shape_event_destinations::de_event_destinations(tokens)?);
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

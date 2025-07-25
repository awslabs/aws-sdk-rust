// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_invoke_code_interpreter_http_response(
    response: &mut ::aws_smithy_runtime_api::http::Response,
) -> std::result::Result<
    crate::operation::invoke_code_interpreter::InvokeCodeInterpreterOutput,
    crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError,
> {
    let mut _response_body = ::aws_smithy_types::body::SdkBody::taken();
    std::mem::swap(&mut _response_body, response.body_mut());
    let _response_body = &mut _response_body;

    let _response_status = response.status().as_u16();
    let _response_headers = response.headers();
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::invoke_code_interpreter::builders::InvokeCodeInterpreterOutputBuilder::default();
        output = output.set_session_id(
            crate::protocol_serde::shape_invoke_code_interpreter_output::de_session_id_header(_response_headers).map_err(|_| {
                crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::unhandled(
                    "Failed to parse sessionId from header `x-amzn-code-interpreter-session-id",
                )
            })?,
        );
        output = output.set_stream(Some(crate::protocol_serde::shape_invoke_code_interpreter_output::de_stream_payload(
            _response_body,
        )?));
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output
            .build()
            .map_err(crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::unhandled)?
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_invoke_code_interpreter_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::invoke_code_interpreter::InvokeCodeInterpreterOutput,
    crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "ValidationException" => crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::ValidationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationExceptionBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_json_err(_response_body, output)
                    .map_err(crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::validation_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::unhandled)?
            };
            tmp
        }),
        "ResourceNotFoundException" => crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::ResourceNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "AccessDeniedException" => crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::AccessDeniedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AccessDeniedExceptionBuilder::default();
                output = crate::protocol_serde::shape_access_denied_exception::de_access_denied_exception_json_err(_response_body, output)
                    .map_err(crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ServiceQuotaExceededException" => crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::ServiceQuotaExceededException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ServiceQuotaExceededExceptionBuilder::default();
                output = crate::protocol_serde::shape_service_quota_exceeded_exception::de_service_quota_exceeded_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ConflictException" => crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::ConflictException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ConflictExceptionBuilder::default();
                output = crate::protocol_serde::shape_conflict_exception::de_conflict_exception_json_err(_response_body, output)
                    .map_err(crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalServerException" => crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::InternalServerException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_server_exception::de_internal_server_exception_json_err(_response_body, output)
                    .map_err(crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ThrottlingException" => crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::ThrottlingException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingExceptionBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(_response_body, output)
                    .map_err(crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::invoke_code_interpreter::InvokeCodeInterpreterError::generic(generic),
    })
}

pub fn ser_invoke_code_interpreter_headers(
    input: &crate::operation::invoke_code_interpreter::InvokeCodeInterpreterInput,
    mut builder: ::http::request::Builder,
) -> std::result::Result<::http::request::Builder, ::aws_smithy_types::error::operation::BuildError> {
    if let ::std::option::Option::Some(inner_1) = &input.session_id {
        let formatted_2 = inner_1.as_str();
        let header_value = formatted_2;
        let header_value: ::http::HeaderValue = header_value.parse().map_err(|err| {
            ::aws_smithy_types::error::operation::BuildError::invalid_field(
                "session_id",
                format!("`{}` cannot be used as a header value: {}", &header_value, err),
            )
        })?;
        builder = builder.header("x-amzn-code-interpreter-session-id", header_value);
    }
    Ok(builder)
}

pub fn ser_invoke_code_interpreter_input(
    input: &crate::operation::invoke_code_interpreter::InvokeCodeInterpreterInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_invoke_code_interpreter_input::ser_invoke_code_interpreter_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

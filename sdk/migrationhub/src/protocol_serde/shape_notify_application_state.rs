// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_notify_application_state_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::notify_application_state::NotifyApplicationStateOutput,
    crate::operation::notify_application_state::NotifyApplicationStateError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => {
            return Err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled(
                generic,
            ))
        }
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AccessDeniedException" => crate::operation::notify_application_state::NotifyApplicationStateError::AccessDeniedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AccessDeniedExceptionBuilder::default();
                output = crate::protocol_serde::shape_access_denied_exception::de_access_denied_exception_json_err(_response_body, output)
                    .map_err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "DryRunOperation" => crate::operation::notify_application_state::NotifyApplicationStateError::DryRunOperation({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::DryRunOperationBuilder::default();
                output = crate::protocol_serde::shape_dry_run_operation::de_dry_run_operation_json_err(_response_body, output)
                    .map_err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "HomeRegionNotSetException" => crate::operation::notify_application_state::NotifyApplicationStateError::HomeRegionNotSetException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::HomeRegionNotSetExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_home_region_not_set_exception::de_home_region_not_set_exception_json_err(_response_body, output)
                        .map_err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalServerError" => crate::operation::notify_application_state::NotifyApplicationStateError::InternalServerError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerErrorBuilder::default();
                output = crate::protocol_serde::shape_internal_server_error::de_internal_server_error_json_err(_response_body, output)
                    .map_err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidInputException" => crate::operation::notify_application_state::NotifyApplicationStateError::InvalidInputException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidInputExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_input_exception::de_invalid_input_exception_json_err(_response_body, output)
                    .map_err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "PolicyErrorException" => crate::operation::notify_application_state::NotifyApplicationStateError::PolicyErrorException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::PolicyErrorExceptionBuilder::default();
                output = crate::protocol_serde::shape_policy_error_exception::de_policy_error_exception_json_err(_response_body, output)
                    .map_err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceNotFoundException" => crate::operation::notify_application_state::NotifyApplicationStateError::ResourceNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ServiceUnavailableException" => crate::operation::notify_application_state::NotifyApplicationStateError::ServiceUnavailableException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ServiceUnavailableExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_service_unavailable_exception::de_service_unavailable_exception_json_err(_response_body, output)
                        .map_err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ThrottlingException" => crate::operation::notify_application_state::NotifyApplicationStateError::ThrottlingException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingExceptionBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(_response_body, output)
                    .map_err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::throttling_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled)?
            };
            tmp
        }),
        "UnauthorizedOperation" => crate::operation::notify_application_state::NotifyApplicationStateError::UnauthorizedOperation({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::UnauthorizedOperationBuilder::default();
                output = crate::protocol_serde::shape_unauthorized_operation::de_unauthorized_operation_json_err(_response_body, output)
                    .map_err(crate::operation::notify_application_state::NotifyApplicationStateError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::notify_application_state::NotifyApplicationStateError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_notify_application_state_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::notify_application_state::NotifyApplicationStateOutput,
    crate::operation::notify_application_state::NotifyApplicationStateError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::notify_application_state::builders::NotifyApplicationStateOutputBuilder::default();
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_notify_application_state_input(
    input: &crate::operation::notify_application_state::NotifyApplicationStateInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_notify_application_state_input::ser_notify_application_state_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

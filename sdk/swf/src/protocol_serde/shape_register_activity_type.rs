// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_register_activity_type_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::register_activity_type::RegisterActivityTypeOutput,
    crate::operation::register_activity_type::RegisterActivityTypeError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::register_activity_type::RegisterActivityTypeError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::register_activity_type::RegisterActivityTypeError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "LimitExceededFault" => crate::operation::register_activity_type::RegisterActivityTypeError::LimitExceededFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::LimitExceededFaultBuilder::default();
                output = crate::protocol_serde::shape_limit_exceeded_fault::de_limit_exceeded_fault_json_err(_response_body, output)
                    .map_err(crate::operation::register_activity_type::RegisterActivityTypeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "OperationNotPermittedFault" => crate::operation::register_activity_type::RegisterActivityTypeError::OperationNotPermittedFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::OperationNotPermittedFaultBuilder::default();
                output =
                    crate::protocol_serde::shape_operation_not_permitted_fault::de_operation_not_permitted_fault_json_err(_response_body, output)
                        .map_err(crate::operation::register_activity_type::RegisterActivityTypeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TypeAlreadyExistsFault" => crate::operation::register_activity_type::RegisterActivityTypeError::TypeAlreadyExistsFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::TypeAlreadyExistsFaultBuilder::default();
                output = crate::protocol_serde::shape_type_already_exists_fault::de_type_already_exists_fault_json_err(_response_body, output)
                    .map_err(crate::operation::register_activity_type::RegisterActivityTypeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "UnknownResourceFault" => crate::operation::register_activity_type::RegisterActivityTypeError::UnknownResourceFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::UnknownResourceFaultBuilder::default();
                output = crate::protocol_serde::shape_unknown_resource_fault::de_unknown_resource_fault_json_err(_response_body, output)
                    .map_err(crate::operation::register_activity_type::RegisterActivityTypeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::register_activity_type::RegisterActivityTypeError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_register_activity_type_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::register_activity_type::RegisterActivityTypeOutput,
    crate::operation::register_activity_type::RegisterActivityTypeError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::register_activity_type::builders::RegisterActivityTypeOutputBuilder::default();
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_register_activity_type_input(
    input: &crate::operation::register_activity_type::RegisterActivityTypeInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_register_activity_type_input::ser_register_activity_type_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

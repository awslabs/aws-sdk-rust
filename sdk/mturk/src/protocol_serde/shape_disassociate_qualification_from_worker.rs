// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_disassociate_qualification_from_worker_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::disassociate_qualification_from_worker::DisassociateQualificationFromWorkerOutput,
    crate::operation::disassociate_qualification_from_worker::DisassociateQualificationFromWorkerError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::disassociate_qualification_from_worker::DisassociateQualificationFromWorkerError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::disassociate_qualification_from_worker::DisassociateQualificationFromWorkerError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "RequestError" => crate::operation::disassociate_qualification_from_worker::DisassociateQualificationFromWorkerError::RequestError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::RequestErrorBuilder::default();
                output = crate::protocol_serde::shape_request_error::de_request_error_json_err(_response_body, output)
                    .map_err(crate::operation::disassociate_qualification_from_worker::DisassociateQualificationFromWorkerError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ServiceFault" => crate::operation::disassociate_qualification_from_worker::DisassociateQualificationFromWorkerError::ServiceFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ServiceFaultBuilder::default();
                output = crate::protocol_serde::shape_service_fault::de_service_fault_json_err(_response_body, output)
                    .map_err(crate::operation::disassociate_qualification_from_worker::DisassociateQualificationFromWorkerError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::disassociate_qualification_from_worker::DisassociateQualificationFromWorkerError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_disassociate_qualification_from_worker_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::disassociate_qualification_from_worker::DisassociateQualificationFromWorkerOutput,
    crate::operation::disassociate_qualification_from_worker::DisassociateQualificationFromWorkerError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output =
            crate::operation::disassociate_qualification_from_worker::builders::DisassociateQualificationFromWorkerOutputBuilder::default();
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_disassociate_qualification_from_worker_input(
    input: &crate::operation::disassociate_qualification_from_worker::DisassociateQualificationFromWorkerInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_disassociate_qualification_from_worker_input::ser_disassociate_qualification_from_worker_input_input(
        &mut object,
        input,
    )?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

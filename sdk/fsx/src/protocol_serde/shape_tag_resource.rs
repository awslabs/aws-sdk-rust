// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_tag_resource_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::tag_resource::TagResourceOutput, crate::operation::tag_resource::TagResourceError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::tag_resource::TagResourceError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::tag_resource::TagResourceError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "BadRequest" => crate::operation::tag_resource::TagResourceError::BadRequest({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::BadRequestBuilder::default();
                output = crate::protocol_serde::shape_bad_request::de_bad_request_json_err(_response_body, output)
                    .map_err(crate::operation::tag_resource::TagResourceError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalServerError" => crate::operation::tag_resource::TagResourceError::InternalServerError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerErrorBuilder::default();
                output = crate::protocol_serde::shape_internal_server_error::de_internal_server_error_json_err(_response_body, output)
                    .map_err(crate::operation::tag_resource::TagResourceError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "NotServiceResourceError" => crate::operation::tag_resource::TagResourceError::NotServiceResourceError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::NotServiceResourceErrorBuilder::default();
                output = crate::protocol_serde::shape_not_service_resource_error::de_not_service_resource_error_json_err(_response_body, output)
                    .map_err(crate::operation::tag_resource::TagResourceError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::not_service_resource_error_correct_errors(output).build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceDoesNotSupportTagging" => crate::operation::tag_resource::TagResourceError::ResourceDoesNotSupportTagging({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceDoesNotSupportTaggingBuilder::default();
                output = crate::protocol_serde::shape_resource_does_not_support_tagging::de_resource_does_not_support_tagging_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::tag_resource::TagResourceError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::resource_does_not_support_tagging_correct_errors(output).build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceNotFound" => crate::operation::tag_resource::TagResourceError::ResourceNotFound({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found::de_resource_not_found_json_err(_response_body, output)
                    .map_err(crate::operation::tag_resource::TagResourceError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::resource_not_found_correct_errors(output).build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::tag_resource::TagResourceError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_tag_resource_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::tag_resource::TagResourceOutput, crate::operation::tag_resource::TagResourceError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::tag_resource::builders::TagResourceOutputBuilder::default();
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_tag_resource_input(
    input: &crate::operation::tag_resource::TagResourceInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_tag_resource_input::ser_tag_resource_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

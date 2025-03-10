// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_define_expression_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::define_expression::DefineExpressionOutput, crate::operation::define_expression::DefineExpressionError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::define_expression::DefineExpressionError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::define_expression::DefineExpressionError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "BaseException" => crate::operation::define_expression::DefineExpressionError::BaseException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::BaseExceptionBuilder::default();
                output = crate::protocol_serde::shape_base_exception::de_base_exception_xml_err(_response_body, output)
                    .map_err(crate::operation::define_expression::DefineExpressionError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalException" => crate::operation::define_expression::DefineExpressionError::InternalException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_exception::de_internal_exception_xml_err(_response_body, output)
                    .map_err(crate::operation::define_expression::DefineExpressionError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidType" => crate::operation::define_expression::DefineExpressionError::InvalidTypeException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidTypeExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_type_exception::de_invalid_type_exception_xml_err(_response_body, output)
                    .map_err(crate::operation::define_expression::DefineExpressionError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "LimitExceeded" => crate::operation::define_expression::DefineExpressionError::LimitExceededException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::LimitExceededExceptionBuilder::default();
                output = crate::protocol_serde::shape_limit_exceeded_exception::de_limit_exceeded_exception_xml_err(_response_body, output)
                    .map_err(crate::operation::define_expression::DefineExpressionError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceNotFound" => crate::operation::define_expression::DefineExpressionError::ResourceNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_xml_err(_response_body, output)
                    .map_err(crate::operation::define_expression::DefineExpressionError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ValidationException" => crate::operation::define_expression::DefineExpressionError::ValidationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationExceptionBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_xml_err(_response_body, output)
                    .map_err(crate::operation::define_expression::DefineExpressionError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::define_expression::DefineExpressionError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_define_expression_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::define_expression::DefineExpressionOutput, crate::operation::define_expression::DefineExpressionError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::define_expression::builders::DefineExpressionOutputBuilder::default();
        output = crate::protocol_serde::shape_define_expression::de_define_expression(_response_body, output)
            .map_err(crate::operation::define_expression::DefineExpressionError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::define_expression_output_output_correct_errors(output).build()
    })
}

#[allow(unused_mut)]
pub fn de_define_expression(
    inp: &[u8],
    mut builder: crate::operation::define_expression::builders::DefineExpressionOutputBuilder,
) -> std::result::Result<crate::operation::define_expression::builders::DefineExpressionOutputBuilder, ::aws_smithy_xml::decode::XmlDecodeError> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("DefineExpressionResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected DefineExpressionResponse got {:?}",
            start_el
        )));
    }
    if let Some(mut result_tag) = decoder.next_tag() {
        let start_el = result_tag.start_el();
        if !(start_el.matches("DefineExpressionResult")) {
            return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
                "invalid result, expected DefineExpressionResult got {:?}",
                start_el
            )));
        }
        while let Some(mut tag) = result_tag.next_tag() {
            match tag.start_el() {
            s if s.matches("Expression") /* Expression com.amazonaws.cloudsearch.synthetic#DefineExpressionOutput$Expression */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_expression_status::de_expression_status(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_expression(var_1);
            }
            ,
            _ => {}
        }
        }
    } else {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom("expected DefineExpressionResult tag"));
    };
    Ok(builder)
}

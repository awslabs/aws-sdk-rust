// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_copy_db_parameter_group_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::copy_db_parameter_group::CopyDbParameterGroupOutput,
    crate::operation::copy_db_parameter_group::CopyDBParameterGroupError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::copy_db_parameter_group::CopyDBParameterGroupError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::copy_db_parameter_group::CopyDBParameterGroupError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "DBParameterGroupAlreadyExists" => {
            crate::operation::copy_db_parameter_group::CopyDBParameterGroupError::DbParameterGroupAlreadyExistsFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DbParameterGroupAlreadyExistsFaultBuilder::default();
                    output =
                        crate::protocol_serde::shape_db_parameter_group_already_exists_fault::de_db_parameter_group_already_exists_fault_xml_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::copy_db_parameter_group::CopyDBParameterGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "DBParameterGroupNotFound" => crate::operation::copy_db_parameter_group::CopyDBParameterGroupError::DbParameterGroupNotFoundFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::DbParameterGroupNotFoundFaultBuilder::default();
                output = crate::protocol_serde::shape_db_parameter_group_not_found_fault::de_db_parameter_group_not_found_fault_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::copy_db_parameter_group::CopyDBParameterGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "DBParameterGroupQuotaExceeded" => {
            crate::operation::copy_db_parameter_group::CopyDBParameterGroupError::DbParameterGroupQuotaExceededFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DbParameterGroupQuotaExceededFaultBuilder::default();
                    output =
                        crate::protocol_serde::shape_db_parameter_group_quota_exceeded_fault::de_db_parameter_group_quota_exceeded_fault_xml_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::copy_db_parameter_group::CopyDBParameterGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        _ => crate::operation::copy_db_parameter_group::CopyDBParameterGroupError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_copy_db_parameter_group_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::copy_db_parameter_group::CopyDbParameterGroupOutput,
    crate::operation::copy_db_parameter_group::CopyDBParameterGroupError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::copy_db_parameter_group::builders::CopyDbParameterGroupOutputBuilder::default();
        output = crate::protocol_serde::shape_copy_db_parameter_group::de_copy_db_parameter_group(_response_body, output)
            .map_err(crate::operation::copy_db_parameter_group::CopyDBParameterGroupError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_copy_db_parameter_group(
    inp: &[u8],
    mut builder: crate::operation::copy_db_parameter_group::builders::CopyDbParameterGroupOutputBuilder,
) -> std::result::Result<
    crate::operation::copy_db_parameter_group::builders::CopyDbParameterGroupOutputBuilder,
    ::aws_smithy_xml::decode::XmlDecodeError,
> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("CopyDBParameterGroupResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected CopyDBParameterGroupResponse got {:?}",
            start_el
        )));
    }
    if let Some(mut result_tag) = decoder.next_tag() {
        let start_el = result_tag.start_el();
        if !(start_el.matches("CopyDBParameterGroupResult")) {
            return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
                "invalid result, expected CopyDBParameterGroupResult got {:?}",
                start_el
            )));
        }
        while let Some(mut tag) = result_tag.next_tag() {
            match tag.start_el() {
            s if s.matches("DBParameterGroup") /* DBParameterGroup com.amazonaws.rds.synthetic#CopyDBParameterGroupOutput$DBParameterGroup */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_db_parameter_group::de_db_parameter_group(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_db_parameter_group(var_1);
            }
            ,
            _ => {}
        }
        }
    } else {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(
            "expected CopyDBParameterGroupResult tag",
        ));
    };
    Ok(builder)
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_authorize_db_security_group_ingress_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::authorize_db_security_group_ingress::AuthorizeDbSecurityGroupIngressOutput,
    crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AuthorizationAlreadyExists" => {
            crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError::AuthorizationAlreadyExistsFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::AuthorizationAlreadyExistsFaultBuilder::default();
                    output = crate::protocol_serde::shape_authorization_already_exists_fault::de_authorization_already_exists_fault_xml_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "AuthorizationQuotaExceeded" => {
            crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError::AuthorizationQuotaExceededFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::AuthorizationQuotaExceededFaultBuilder::default();
                    output = crate::protocol_serde::shape_authorization_quota_exceeded_fault::de_authorization_quota_exceeded_fault_xml_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "DBSecurityGroupNotFound" => {
            crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError::DbSecurityGroupNotFoundFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DbSecurityGroupNotFoundFaultBuilder::default();
                    output = crate::protocol_serde::shape_db_security_group_not_found_fault::de_db_security_group_not_found_fault_xml_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidDBSecurityGroupState" => {
            crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError::InvalidDbSecurityGroupStateFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidDbSecurityGroupStateFaultBuilder::default();
                    output = crate::protocol_serde::shape_invalid_db_security_group_state_fault::de_invalid_db_security_group_state_fault_xml_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        _ => crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_authorize_db_security_group_ingress_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::authorize_db_security_group_ingress::AuthorizeDbSecurityGroupIngressOutput,
    crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::authorize_db_security_group_ingress::builders::AuthorizeDbSecurityGroupIngressOutputBuilder::default();
        output = crate::protocol_serde::shape_authorize_db_security_group_ingress::de_authorize_db_security_group_ingress(_response_body, output)
            .map_err(crate::operation::authorize_db_security_group_ingress::AuthorizeDBSecurityGroupIngressError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_authorize_db_security_group_ingress(
    inp: &[u8],
    mut builder: crate::operation::authorize_db_security_group_ingress::builders::AuthorizeDbSecurityGroupIngressOutputBuilder,
) -> std::result::Result<
    crate::operation::authorize_db_security_group_ingress::builders::AuthorizeDbSecurityGroupIngressOutputBuilder,
    ::aws_smithy_xml::decode::XmlDecodeError,
> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("AuthorizeDBSecurityGroupIngressResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected AuthorizeDBSecurityGroupIngressResponse got {:?}",
            start_el
        )));
    }
    if let Some(mut result_tag) = decoder.next_tag() {
        let start_el = result_tag.start_el();
        if !(start_el.matches("AuthorizeDBSecurityGroupIngressResult")) {
            return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
                "invalid result, expected AuthorizeDBSecurityGroupIngressResult got {:?}",
                start_el
            )));
        }
        while let Some(mut tag) = result_tag.next_tag() {
            match tag.start_el() {
            s if s.matches("DBSecurityGroup") /* DBSecurityGroup com.amazonaws.rds.synthetic#AuthorizeDBSecurityGroupIngressOutput$DBSecurityGroup */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_db_security_group::de_db_security_group(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_db_security_group(var_1);
            }
            ,
            _ => {}
        }
        }
    } else {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(
            "expected AuthorizeDBSecurityGroupIngressResult tag",
        ));
    };
    Ok(builder)
}

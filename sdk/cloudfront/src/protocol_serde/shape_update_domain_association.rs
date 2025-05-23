// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_update_domain_association_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::update_domain_association::UpdateDomainAssociationOutput,
    crate::operation::update_domain_association::UpdateDomainAssociationError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::update_domain_association::UpdateDomainAssociationError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => {
            return Err(crate::operation::update_domain_association::UpdateDomainAssociationError::unhandled(
                generic,
            ))
        }
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AccessDenied" => crate::operation::update_domain_association::UpdateDomainAssociationError::AccessDenied({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AccessDeniedBuilder::default();
                output = crate::protocol_serde::shape_access_denied::de_access_denied_xml_err(_response_body, output)
                    .map_err(crate::operation::update_domain_association::UpdateDomainAssociationError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "EntityNotFound" => crate::operation::update_domain_association::UpdateDomainAssociationError::EntityNotFound({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::EntityNotFoundBuilder::default();
                output = crate::protocol_serde::shape_entity_not_found::de_entity_not_found_xml_err(_response_body, output)
                    .map_err(crate::operation::update_domain_association::UpdateDomainAssociationError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "IllegalUpdate" => crate::operation::update_domain_association::UpdateDomainAssociationError::IllegalUpdate({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::IllegalUpdateBuilder::default();
                output = crate::protocol_serde::shape_illegal_update::de_illegal_update_xml_err(_response_body, output)
                    .map_err(crate::operation::update_domain_association::UpdateDomainAssociationError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidArgument" => crate::operation::update_domain_association::UpdateDomainAssociationError::InvalidArgument({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidArgumentBuilder::default();
                output = crate::protocol_serde::shape_invalid_argument::de_invalid_argument_xml_err(_response_body, output)
                    .map_err(crate::operation::update_domain_association::UpdateDomainAssociationError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidIfMatchVersion" => crate::operation::update_domain_association::UpdateDomainAssociationError::InvalidIfMatchVersion({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidIfMatchVersionBuilder::default();
                output = crate::protocol_serde::shape_invalid_if_match_version::de_invalid_if_match_version_xml_err(_response_body, output)
                    .map_err(crate::operation::update_domain_association::UpdateDomainAssociationError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "PreconditionFailed" => crate::operation::update_domain_association::UpdateDomainAssociationError::PreconditionFailed({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::PreconditionFailedBuilder::default();
                output = crate::protocol_serde::shape_precondition_failed::de_precondition_failed_xml_err(_response_body, output)
                    .map_err(crate::operation::update_domain_association::UpdateDomainAssociationError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::update_domain_association::UpdateDomainAssociationError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_update_domain_association_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::update_domain_association::UpdateDomainAssociationOutput,
    crate::operation::update_domain_association::UpdateDomainAssociationError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::update_domain_association::builders::UpdateDomainAssociationOutputBuilder::default();
        output = crate::protocol_serde::shape_update_domain_association::de_update_domain_association(_response_body, output)
            .map_err(crate::operation::update_domain_association::UpdateDomainAssociationError::unhandled)?;
        output = output.set_e_tag(
            crate::protocol_serde::shape_update_domain_association_output::de_e_tag_header(_response_headers).map_err(|_| {
                crate::operation::update_domain_association::UpdateDomainAssociationError::unhandled("Failed to parse ETag from header `ETag")
            })?,
        );
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_update_domain_association_headers(
    input: &crate::operation::update_domain_association::UpdateDomainAssociationInput,
    mut builder: ::http::request::Builder,
) -> std::result::Result<::http::request::Builder, ::aws_smithy_types::error::operation::BuildError> {
    if let ::std::option::Option::Some(inner_1) = &input.if_match {
        let formatted_2 = inner_1.as_str();
        let header_value = formatted_2;
        let header_value: ::http::HeaderValue = header_value.parse().map_err(|err| {
            ::aws_smithy_types::error::operation::BuildError::invalid_field(
                "if_match",
                format!("`{}` cannot be used as a header value: {}", &header_value, err),
            )
        })?;
        builder = builder.header("If-Match", header_value);
    }
    Ok(builder)
}

pub fn ser_update_domain_association_op_input(
    input: &crate::operation::update_domain_association::UpdateDomainAssociationInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    {
        let mut writer = ::aws_smithy_xml::encode::XmlWriter::new(&mut out);
        #[allow(unused_mut)]
        let mut root = writer
            .start_el("UpdateDomainAssociationRequest")
            .write_ns("http://cloudfront.amazonaws.com/doc/2020-05-31/", None);
        crate::protocol_serde::shape_update_domain_association_input::ser_update_domain_association_input_input_input(input, root)?
    }
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

#[allow(unused_mut)]
pub fn de_update_domain_association(
    inp: &[u8],
    mut builder: crate::operation::update_domain_association::builders::UpdateDomainAssociationOutputBuilder,
) -> std::result::Result<
    crate::operation::update_domain_association::builders::UpdateDomainAssociationOutputBuilder,
    ::aws_smithy_xml::decode::XmlDecodeError,
> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !start_el.matches("UpdateDomainAssociationResult") {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "encountered invalid XML root: expected UpdateDomainAssociationResult but got {:?}. This is likely a bug in the SDK.",
            start_el
        )));
    }
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("ResourceId") /* ResourceId com.amazonaws.cloudfront.synthetic#UpdateDomainAssociationOutput$ResourceId */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_resource_id(var_3);
            }
            ,
            s if s.matches("Domain") /* Domain com.amazonaws.cloudfront.synthetic#UpdateDomainAssociationOutput$Domain */ =>  {
                let var_4 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_domain(var_4);
            }
            ,
            _ => {}
        }
    }
    Ok(builder)
}

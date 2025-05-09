// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_get_ssh_public_key_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::get_ssh_public_key::GetSshPublicKeyOutput, crate::operation::get_ssh_public_key::GetSSHPublicKeyError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::get_ssh_public_key::GetSSHPublicKeyError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::get_ssh_public_key::GetSSHPublicKeyError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "NoSuchEntity" => crate::operation::get_ssh_public_key::GetSSHPublicKeyError::NoSuchEntityException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::NoSuchEntityExceptionBuilder::default();
                output = crate::protocol_serde::shape_no_such_entity_exception::de_no_such_entity_exception_xml_err(_response_body, output)
                    .map_err(crate::operation::get_ssh_public_key::GetSSHPublicKeyError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "UnrecognizedPublicKeyEncoding" => {
            crate::operation::get_ssh_public_key::GetSSHPublicKeyError::UnrecognizedPublicKeyEncodingException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::UnrecognizedPublicKeyEncodingExceptionBuilder::default();
                    output = crate::protocol_serde::shape_unrecognized_public_key_encoding_exception::de_unrecognized_public_key_encoding_exception_xml_err(_response_body, output).map_err(crate::operation::get_ssh_public_key::GetSSHPublicKeyError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        _ => crate::operation::get_ssh_public_key::GetSSHPublicKeyError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_get_ssh_public_key_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::get_ssh_public_key::GetSshPublicKeyOutput, crate::operation::get_ssh_public_key::GetSSHPublicKeyError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::get_ssh_public_key::builders::GetSshPublicKeyOutputBuilder::default();
        output = crate::protocol_serde::shape_get_ssh_public_key::de_get_ssh_public_key(_response_body, output)
            .map_err(crate::operation::get_ssh_public_key::GetSSHPublicKeyError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_get_ssh_public_key(
    inp: &[u8],
    mut builder: crate::operation::get_ssh_public_key::builders::GetSshPublicKeyOutputBuilder,
) -> std::result::Result<crate::operation::get_ssh_public_key::builders::GetSshPublicKeyOutputBuilder, ::aws_smithy_xml::decode::XmlDecodeError> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("GetSSHPublicKeyResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected GetSSHPublicKeyResponse got {:?}",
            start_el
        )));
    }
    if let Some(mut result_tag) = decoder.next_tag() {
        let start_el = result_tag.start_el();
        if !(start_el.matches("GetSSHPublicKeyResult")) {
            return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
                "invalid result, expected GetSSHPublicKeyResult got {:?}",
                start_el
            )));
        }
        while let Some(mut tag) = result_tag.next_tag() {
            match tag.start_el() {
            s if s.matches("SSHPublicKey") /* SSHPublicKey com.amazonaws.iam.synthetic#GetSSHPublicKeyOutput$SSHPublicKey */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_ssh_public_key::de_ssh_public_key(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_ssh_public_key(var_1);
            }
            ,
            _ => {}
        }
        }
    } else {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom("expected GetSSHPublicKeyResult tag"));
    };
    Ok(builder)
}

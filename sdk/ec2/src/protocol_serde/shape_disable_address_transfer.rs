// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_disable_address_transfer_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::disable_address_transfer::DisableAddressTransferOutput,
    crate::operation::disable_address_transfer::DisableAddressTransferError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::disable_address_transfer::DisableAddressTransferError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    Err(crate::operation::disable_address_transfer::DisableAddressTransferError::generic(generic))
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_disable_address_transfer_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::disable_address_transfer::DisableAddressTransferOutput,
    crate::operation::disable_address_transfer::DisableAddressTransferError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::disable_address_transfer::builders::DisableAddressTransferOutputBuilder::default();
        output = crate::protocol_serde::shape_disable_address_transfer::de_disable_address_transfer(_response_body, output)
            .map_err(crate::operation::disable_address_transfer::DisableAddressTransferError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_disable_address_transfer(
    inp: &[u8],
    mut builder: crate::operation::disable_address_transfer::builders::DisableAddressTransferOutputBuilder,
) -> std::result::Result<
    crate::operation::disable_address_transfer::builders::DisableAddressTransferOutputBuilder,
    ::aws_smithy_xml::decode::XmlDecodeError,
> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("DisableAddressTransferResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected DisableAddressTransferResponse got {:?}",
            start_el
        )));
    }
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("addressTransfer") /* AddressTransfer com.amazonaws.ec2.synthetic#DisableAddressTransferOutput$AddressTransfer */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_address_transfer::de_address_transfer(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_address_transfer(var_1);
            }
            ,
            _ => {}
        }
    }
    Ok(builder)
}

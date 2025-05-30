// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_modify_ipam_resource_cidr_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::modify_ipam_resource_cidr::ModifyIpamResourceCidrOutput,
    crate::operation::modify_ipam_resource_cidr::ModifyIpamResourceCidrError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::modify_ipam_resource_cidr::ModifyIpamResourceCidrError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    Err(crate::operation::modify_ipam_resource_cidr::ModifyIpamResourceCidrError::generic(generic))
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_modify_ipam_resource_cidr_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::modify_ipam_resource_cidr::ModifyIpamResourceCidrOutput,
    crate::operation::modify_ipam_resource_cidr::ModifyIpamResourceCidrError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::modify_ipam_resource_cidr::builders::ModifyIpamResourceCidrOutputBuilder::default();
        output = crate::protocol_serde::shape_modify_ipam_resource_cidr::de_modify_ipam_resource_cidr(_response_body, output)
            .map_err(crate::operation::modify_ipam_resource_cidr::ModifyIpamResourceCidrError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_modify_ipam_resource_cidr(
    inp: &[u8],
    mut builder: crate::operation::modify_ipam_resource_cidr::builders::ModifyIpamResourceCidrOutputBuilder,
) -> std::result::Result<
    crate::operation::modify_ipam_resource_cidr::builders::ModifyIpamResourceCidrOutputBuilder,
    ::aws_smithy_xml::decode::XmlDecodeError,
> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("ModifyIpamResourceCidrResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected ModifyIpamResourceCidrResponse got {:?}",
            start_el
        )));
    }
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("ipamResourceCidr") /* IpamResourceCidr com.amazonaws.ec2.synthetic#ModifyIpamResourceCidrOutput$IpamResourceCidr */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_ipam_resource_cidr::de_ipam_resource_cidr(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_ipam_resource_cidr(var_1);
            }
            ,
            _ => {}
        }
    }
    Ok(builder)
}

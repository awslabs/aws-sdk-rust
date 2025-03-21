// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_dhcp_options_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::describe_dhcp_options::DescribeDhcpOptionsOutput,
    crate::operation::describe_dhcp_options::DescribeDhcpOptionsError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::describe_dhcp_options::DescribeDhcpOptionsError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    Err(crate::operation::describe_dhcp_options::DescribeDhcpOptionsError::generic(generic))
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_dhcp_options_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::describe_dhcp_options::DescribeDhcpOptionsOutput,
    crate::operation::describe_dhcp_options::DescribeDhcpOptionsError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::describe_dhcp_options::builders::DescribeDhcpOptionsOutputBuilder::default();
        output = crate::protocol_serde::shape_describe_dhcp_options::de_describe_dhcp_options(_response_body, output)
            .map_err(crate::operation::describe_dhcp_options::DescribeDhcpOptionsError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_describe_dhcp_options(
    inp: &[u8],
    mut builder: crate::operation::describe_dhcp_options::builders::DescribeDhcpOptionsOutputBuilder,
) -> std::result::Result<crate::operation::describe_dhcp_options::builders::DescribeDhcpOptionsOutputBuilder, ::aws_smithy_xml::decode::XmlDecodeError>
{
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("DescribeDhcpOptionsResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected DescribeDhcpOptionsResponse got {:?}",
            start_el
        )));
    }
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("nextToken") /* NextToken com.amazonaws.ec2.synthetic#DescribeDhcpOptionsOutput$NextToken */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_next_token(var_1);
            }
            ,
            s if s.matches("dhcpOptionsSet") /* DhcpOptions com.amazonaws.ec2.synthetic#DescribeDhcpOptionsOutput$DhcpOptions */ =>  {
                let var_2 =
                    Some(
                        crate::protocol_serde::shape_dhcp_options_list::de_dhcp_options_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_dhcp_options(var_2);
            }
            ,
            _ => {}
        }
    }
    Ok(builder)
}

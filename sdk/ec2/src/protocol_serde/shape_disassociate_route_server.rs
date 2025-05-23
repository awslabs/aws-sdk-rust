// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_disassociate_route_server_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::disassociate_route_server::DisassociateRouteServerOutput,
    crate::operation::disassociate_route_server::DisassociateRouteServerError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::disassociate_route_server::DisassociateRouteServerError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    Err(crate::operation::disassociate_route_server::DisassociateRouteServerError::generic(
        generic,
    ))
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_disassociate_route_server_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::disassociate_route_server::DisassociateRouteServerOutput,
    crate::operation::disassociate_route_server::DisassociateRouteServerError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::disassociate_route_server::builders::DisassociateRouteServerOutputBuilder::default();
        output = crate::protocol_serde::shape_disassociate_route_server::de_disassociate_route_server(_response_body, output)
            .map_err(crate::operation::disassociate_route_server::DisassociateRouteServerError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_disassociate_route_server(
    inp: &[u8],
    mut builder: crate::operation::disassociate_route_server::builders::DisassociateRouteServerOutputBuilder,
) -> std::result::Result<
    crate::operation::disassociate_route_server::builders::DisassociateRouteServerOutputBuilder,
    ::aws_smithy_xml::decode::XmlDecodeError,
> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("DisassociateRouteServerResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected DisassociateRouteServerResponse got {:?}",
            start_el
        )));
    }
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("routeServerAssociation") /* RouteServerAssociation com.amazonaws.ec2.synthetic#DisassociateRouteServerOutput$RouteServerAssociation */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_route_server_association::de_route_server_association(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_route_server_association(var_1);
            }
            ,
            _ => {}
        }
    }
    Ok(builder)
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_delete_instance_connect_endpoint_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::delete_instance_connect_endpoint::DeleteInstanceConnectEndpointOutput,
    crate::operation::delete_instance_connect_endpoint::DeleteInstanceConnectEndpointError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::delete_instance_connect_endpoint::DeleteInstanceConnectEndpointError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    Err(crate::operation::delete_instance_connect_endpoint::DeleteInstanceConnectEndpointError::generic(generic))
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_delete_instance_connect_endpoint_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::delete_instance_connect_endpoint::DeleteInstanceConnectEndpointOutput,
    crate::operation::delete_instance_connect_endpoint::DeleteInstanceConnectEndpointError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::delete_instance_connect_endpoint::builders::DeleteInstanceConnectEndpointOutputBuilder::default();
        output = crate::protocol_serde::shape_delete_instance_connect_endpoint::de_delete_instance_connect_endpoint(_response_body, output)
            .map_err(crate::operation::delete_instance_connect_endpoint::DeleteInstanceConnectEndpointError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_delete_instance_connect_endpoint(
    inp: &[u8],
    mut builder: crate::operation::delete_instance_connect_endpoint::builders::DeleteInstanceConnectEndpointOutputBuilder,
) -> std::result::Result<
    crate::operation::delete_instance_connect_endpoint::builders::DeleteInstanceConnectEndpointOutputBuilder,
    ::aws_smithy_xml::decode::XmlDecodeError,
> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("DeleteInstanceConnectEndpointResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected DeleteInstanceConnectEndpointResponse got {:?}",
            start_el
        )));
    }
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("instanceConnectEndpoint") /* InstanceConnectEndpoint com.amazonaws.ec2.synthetic#DeleteInstanceConnectEndpointOutput$InstanceConnectEndpoint */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_ec2_instance_connect_endpoint::de_ec2_instance_connect_endpoint(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_instance_connect_endpoint(var_1);
            }
            ,
            _ => {}
        }
    }
    Ok(builder)
}

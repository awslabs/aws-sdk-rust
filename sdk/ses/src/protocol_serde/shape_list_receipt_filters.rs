// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_list_receipt_filters_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::list_receipt_filters::ListReceiptFiltersOutput,
    crate::operation::list_receipt_filters::ListReceiptFiltersError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::list_receipt_filters::ListReceiptFiltersError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    Err(crate::operation::list_receipt_filters::ListReceiptFiltersError::generic(generic))
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_list_receipt_filters_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::list_receipt_filters::ListReceiptFiltersOutput,
    crate::operation::list_receipt_filters::ListReceiptFiltersError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::list_receipt_filters::builders::ListReceiptFiltersOutputBuilder::default();
        output = crate::protocol_serde::shape_list_receipt_filters::de_list_receipt_filters(_response_body, output)
            .map_err(crate::operation::list_receipt_filters::ListReceiptFiltersError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_list_receipt_filters(
    inp: &[u8],
    mut builder: crate::operation::list_receipt_filters::builders::ListReceiptFiltersOutputBuilder,
) -> std::result::Result<crate::operation::list_receipt_filters::builders::ListReceiptFiltersOutputBuilder, ::aws_smithy_xml::decode::XmlDecodeError>
{
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("ListReceiptFiltersResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected ListReceiptFiltersResponse got {:?}",
            start_el
        )));
    }
    if let Some(mut result_tag) = decoder.next_tag() {
        let start_el = result_tag.start_el();
        if !(start_el.matches("ListReceiptFiltersResult")) {
            return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
                "invalid result, expected ListReceiptFiltersResult got {:?}",
                start_el
            )));
        }
        while let Some(mut tag) = result_tag.next_tag() {
            match tag.start_el() {
            s if s.matches("Filters") /* Filters com.amazonaws.ses.synthetic#ListReceiptFiltersOutput$Filters */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_receipt_filter_list::de_receipt_filter_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_filters(var_1);
            }
            ,
            _ => {}
        }
        }
    } else {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom("expected ListReceiptFiltersResult tag"));
    };
    Ok(builder)
}

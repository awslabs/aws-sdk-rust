// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_list_stack_refactors_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::list_stack_refactors::ListStackRefactorsOutput,
    crate::operation::list_stack_refactors::ListStackRefactorsError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::list_stack_refactors::ListStackRefactorsError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    Err(crate::operation::list_stack_refactors::ListStackRefactorsError::generic(generic))
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_list_stack_refactors_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::list_stack_refactors::ListStackRefactorsOutput,
    crate::operation::list_stack_refactors::ListStackRefactorsError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::list_stack_refactors::builders::ListStackRefactorsOutputBuilder::default();
        output = crate::protocol_serde::shape_list_stack_refactors::de_list_stack_refactors(_response_body, output)
            .map_err(crate::operation::list_stack_refactors::ListStackRefactorsError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::list_stack_refactors_output_output_correct_errors(output).build()
    })
}

#[allow(unused_mut)]
pub fn de_list_stack_refactors(
    inp: &[u8],
    mut builder: crate::operation::list_stack_refactors::builders::ListStackRefactorsOutputBuilder,
) -> std::result::Result<crate::operation::list_stack_refactors::builders::ListStackRefactorsOutputBuilder, ::aws_smithy_xml::decode::XmlDecodeError>
{
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("ListStackRefactorsResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected ListStackRefactorsResponse got {:?}",
            start_el
        )));
    }
    if let Some(mut result_tag) = decoder.next_tag() {
        let start_el = result_tag.start_el();
        if !(start_el.matches("ListStackRefactorsResult")) {
            return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
                "invalid result, expected ListStackRefactorsResult got {:?}",
                start_el
            )));
        }
        while let Some(mut tag) = result_tag.next_tag() {
            match tag.start_el() {
            s if s.matches("StackRefactorSummaries") /* StackRefactorSummaries com.amazonaws.cloudformation.synthetic#ListStackRefactorsOutput$StackRefactorSummaries */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_stack_refactor_summaries::de_stack_refactor_summaries(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_stack_refactor_summaries(var_1);
            }
            ,
            s if s.matches("NextToken") /* NextToken com.amazonaws.cloudformation.synthetic#ListStackRefactorsOutput$NextToken */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_next_token(var_2);
            }
            ,
            _ => {}
        }
        }
    } else {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom("expected ListStackRefactorsResult tag"));
    };
    Ok(builder)
}

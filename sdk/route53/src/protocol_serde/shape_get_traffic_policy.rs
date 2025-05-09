// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_get_traffic_policy_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::get_traffic_policy::GetTrafficPolicyOutput, crate::operation::get_traffic_policy::GetTrafficPolicyError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::get_traffic_policy::GetTrafficPolicyError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::get_traffic_policy::GetTrafficPolicyError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "InvalidInput" => crate::operation::get_traffic_policy::GetTrafficPolicyError::InvalidInput({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidInputBuilder::default();
                output = crate::protocol_serde::shape_invalid_input::de_invalid_input_xml_err(_response_body, output)
                    .map_err(crate::operation::get_traffic_policy::GetTrafficPolicyError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "NoSuchTrafficPolicy" => crate::operation::get_traffic_policy::GetTrafficPolicyError::NoSuchTrafficPolicy({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::NoSuchTrafficPolicyBuilder::default();
                output = crate::protocol_serde::shape_no_such_traffic_policy::de_no_such_traffic_policy_xml_err(_response_body, output)
                    .map_err(crate::operation::get_traffic_policy::GetTrafficPolicyError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::get_traffic_policy::GetTrafficPolicyError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_get_traffic_policy_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::get_traffic_policy::GetTrafficPolicyOutput, crate::operation::get_traffic_policy::GetTrafficPolicyError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::get_traffic_policy::builders::GetTrafficPolicyOutputBuilder::default();
        output = crate::protocol_serde::shape_get_traffic_policy::de_get_traffic_policy(_response_body, output)
            .map_err(crate::operation::get_traffic_policy::GetTrafficPolicyError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::get_traffic_policy_output_output_correct_errors(output).build()
    })
}

#[allow(unused_mut)]
pub fn de_get_traffic_policy(
    inp: &[u8],
    mut builder: crate::operation::get_traffic_policy::builders::GetTrafficPolicyOutputBuilder,
) -> std::result::Result<crate::operation::get_traffic_policy::builders::GetTrafficPolicyOutputBuilder, ::aws_smithy_xml::decode::XmlDecodeError> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !start_el.matches("GetTrafficPolicyResponse") {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "encountered invalid XML root: expected GetTrafficPolicyResponse but got {:?}. This is likely a bug in the SDK.",
            start_el
        )));
    }
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("TrafficPolicy") /* TrafficPolicy com.amazonaws.route53.synthetic#GetTrafficPolicyOutput$TrafficPolicy */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_traffic_policy::de_traffic_policy(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_traffic_policy(var_1);
            }
            ,
            _ => {}
        }
    }
    Ok(builder)
}

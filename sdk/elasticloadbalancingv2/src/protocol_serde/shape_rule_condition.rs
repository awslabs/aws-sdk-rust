// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(unused_mut)]
pub fn ser_rule_condition(
    mut writer: ::aws_smithy_query::QueryValueWriter,
    input: &crate::types::RuleCondition,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("Field");
    if let Some(var_2) = &input.field {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("Values");
    if let Some(var_4) = &input.values {
        let mut list_6 = scope_3.start_list(false, None);
        for item_5 in var_4 {
            #[allow(unused_mut)]
            let mut entry_7 = list_6.entry();
            entry_7.string(item_5);
        }
        list_6.finish();
    }
    #[allow(unused_mut)]
    let mut scope_8 = writer.prefix("HostHeaderConfig");
    if let Some(var_9) = &input.host_header_config {
        crate::protocol_serde::shape_host_header_condition_config::ser_host_header_condition_config(scope_8, var_9)?;
    }
    #[allow(unused_mut)]
    let mut scope_10 = writer.prefix("PathPatternConfig");
    if let Some(var_11) = &input.path_pattern_config {
        crate::protocol_serde::shape_path_pattern_condition_config::ser_path_pattern_condition_config(scope_10, var_11)?;
    }
    #[allow(unused_mut)]
    let mut scope_12 = writer.prefix("HttpHeaderConfig");
    if let Some(var_13) = &input.http_header_config {
        crate::protocol_serde::shape_http_header_condition_config::ser_http_header_condition_config(scope_12, var_13)?;
    }
    #[allow(unused_mut)]
    let mut scope_14 = writer.prefix("QueryStringConfig");
    if let Some(var_15) = &input.query_string_config {
        crate::protocol_serde::shape_query_string_condition_config::ser_query_string_condition_config(scope_14, var_15)?;
    }
    #[allow(unused_mut)]
    let mut scope_16 = writer.prefix("HttpRequestMethodConfig");
    if let Some(var_17) = &input.http_request_method_config {
        crate::protocol_serde::shape_http_request_method_condition_config::ser_http_request_method_condition_config(scope_16, var_17)?;
    }
    #[allow(unused_mut)]
    let mut scope_18 = writer.prefix("SourceIpConfig");
    if let Some(var_19) = &input.source_ip_config {
        crate::protocol_serde::shape_source_ip_condition_config::ser_source_ip_condition_config(scope_18, var_19)?;
    }
    Ok(())
}

#[allow(clippy::needless_question_mark)]
pub fn de_rule_condition(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::RuleCondition, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::RuleCondition::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("Field") /* Field com.amazonaws.elasticloadbalancingv2#RuleCondition$Field */ =>  {
                let var_20 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_field(var_20);
            }
            ,
            s if s.matches("Values") /* Values com.amazonaws.elasticloadbalancingv2#RuleCondition$Values */ =>  {
                let var_21 =
                    Some(
                        crate::protocol_serde::shape_list_of_string::de_list_of_string(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_values(var_21);
            }
            ,
            s if s.matches("HostHeaderConfig") /* HostHeaderConfig com.amazonaws.elasticloadbalancingv2#RuleCondition$HostHeaderConfig */ =>  {
                let var_22 =
                    Some(
                        crate::protocol_serde::shape_host_header_condition_config::de_host_header_condition_config(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_host_header_config(var_22);
            }
            ,
            s if s.matches("PathPatternConfig") /* PathPatternConfig com.amazonaws.elasticloadbalancingv2#RuleCondition$PathPatternConfig */ =>  {
                let var_23 =
                    Some(
                        crate::protocol_serde::shape_path_pattern_condition_config::de_path_pattern_condition_config(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_path_pattern_config(var_23);
            }
            ,
            s if s.matches("HttpHeaderConfig") /* HttpHeaderConfig com.amazonaws.elasticloadbalancingv2#RuleCondition$HttpHeaderConfig */ =>  {
                let var_24 =
                    Some(
                        crate::protocol_serde::shape_http_header_condition_config::de_http_header_condition_config(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_http_header_config(var_24);
            }
            ,
            s if s.matches("QueryStringConfig") /* QueryStringConfig com.amazonaws.elasticloadbalancingv2#RuleCondition$QueryStringConfig */ =>  {
                let var_25 =
                    Some(
                        crate::protocol_serde::shape_query_string_condition_config::de_query_string_condition_config(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_query_string_config(var_25);
            }
            ,
            s if s.matches("HttpRequestMethodConfig") /* HttpRequestMethodConfig com.amazonaws.elasticloadbalancingv2#RuleCondition$HttpRequestMethodConfig */ =>  {
                let var_26 =
                    Some(
                        crate::protocol_serde::shape_http_request_method_condition_config::de_http_request_method_condition_config(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_http_request_method_config(var_26);
            }
            ,
            s if s.matches("SourceIpConfig") /* SourceIpConfig com.amazonaws.elasticloadbalancingv2#RuleCondition$SourceIpConfig */ =>  {
                let var_27 =
                    Some(
                        crate::protocol_serde::shape_source_ip_condition_config::de_source_ip_condition_config(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_source_ip_config(var_27);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

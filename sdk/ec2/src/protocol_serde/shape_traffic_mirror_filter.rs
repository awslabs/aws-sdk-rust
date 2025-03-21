// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_traffic_mirror_filter(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::TrafficMirrorFilter, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::TrafficMirrorFilter::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("trafficMirrorFilterId") /* TrafficMirrorFilterId com.amazonaws.ec2#TrafficMirrorFilter$TrafficMirrorFilterId */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_traffic_mirror_filter_id(var_1);
            }
            ,
            s if s.matches("ingressFilterRuleSet") /* IngressFilterRules com.amazonaws.ec2#TrafficMirrorFilter$IngressFilterRules */ =>  {
                let var_2 =
                    Some(
                        crate::protocol_serde::shape_traffic_mirror_filter_rule_list::de_traffic_mirror_filter_rule_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_ingress_filter_rules(var_2);
            }
            ,
            s if s.matches("egressFilterRuleSet") /* EgressFilterRules com.amazonaws.ec2#TrafficMirrorFilter$EgressFilterRules */ =>  {
                let var_3 =
                    Some(
                        crate::protocol_serde::shape_traffic_mirror_filter_rule_list::de_traffic_mirror_filter_rule_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_egress_filter_rules(var_3);
            }
            ,
            s if s.matches("networkServiceSet") /* NetworkServices com.amazonaws.ec2#TrafficMirrorFilter$NetworkServices */ =>  {
                let var_4 =
                    Some(
                        crate::protocol_serde::shape_traffic_mirror_network_service_list::de_traffic_mirror_network_service_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_network_services(var_4);
            }
            ,
            s if s.matches("description") /* Description com.amazonaws.ec2#TrafficMirrorFilter$Description */ =>  {
                let var_5 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_description(var_5);
            }
            ,
            s if s.matches("tagSet") /* Tags com.amazonaws.ec2#TrafficMirrorFilter$Tags */ =>  {
                let var_6 =
                    Some(
                        crate::protocol_serde::shape_tag_list::de_tag_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_tags(var_6);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

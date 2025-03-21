// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_verified_access_endpoint_load_balancer_options(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::VerifiedAccessEndpointLoadBalancerOptions, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::VerifiedAccessEndpointLoadBalancerOptions::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("protocol") /* Protocol com.amazonaws.ec2#VerifiedAccessEndpointLoadBalancerOptions$Protocol */ =>  {
                let var_1 =
                    Some(
                        Result::<crate::types::VerifiedAccessEndpointProtocol, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::VerifiedAccessEndpointProtocol::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_protocol(var_1);
            }
            ,
            s if s.matches("port") /* Port com.amazonaws.ec2#VerifiedAccessEndpointLoadBalancerOptions$Port */ =>  {
                let var_2 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.ec2#VerifiedAccessEndpointPortNumber`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_port(var_2);
            }
            ,
            s if s.matches("loadBalancerArn") /* LoadBalancerArn com.amazonaws.ec2#VerifiedAccessEndpointLoadBalancerOptions$LoadBalancerArn */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_load_balancer_arn(var_3);
            }
            ,
            s if s.matches("subnetIdSet") /* SubnetIds com.amazonaws.ec2#VerifiedAccessEndpointLoadBalancerOptions$SubnetIds */ =>  {
                let var_4 =
                    Some(
                        crate::protocol_serde::shape_verified_access_endpoint_subnet_id_list::de_verified_access_endpoint_subnet_id_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_subnet_ids(var_4);
            }
            ,
            s if s.matches("portRangeSet") /* PortRanges com.amazonaws.ec2#VerifiedAccessEndpointLoadBalancerOptions$PortRanges */ =>  {
                let var_5 =
                    Some(
                        crate::protocol_serde::shape_verified_access_endpoint_port_range_list::de_verified_access_endpoint_port_range_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_port_ranges(var_5);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_vpc_endpoint_association(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::VpcEndpointAssociation, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::VpcEndpointAssociation::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("id") /* Id com.amazonaws.ec2#VpcEndpointAssociation$Id */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_id(var_1);
            }
            ,
            s if s.matches("vpcEndpointId") /* VpcEndpointId com.amazonaws.ec2#VpcEndpointAssociation$VpcEndpointId */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_vpc_endpoint_id(var_2);
            }
            ,
            s if s.matches("serviceNetworkArn") /* ServiceNetworkArn com.amazonaws.ec2#VpcEndpointAssociation$ServiceNetworkArn */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_service_network_arn(var_3);
            }
            ,
            s if s.matches("serviceNetworkName") /* ServiceNetworkName com.amazonaws.ec2#VpcEndpointAssociation$ServiceNetworkName */ =>  {
                let var_4 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_service_network_name(var_4);
            }
            ,
            s if s.matches("associatedResourceAccessibility") /* AssociatedResourceAccessibility com.amazonaws.ec2#VpcEndpointAssociation$AssociatedResourceAccessibility */ =>  {
                let var_5 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_associated_resource_accessibility(var_5);
            }
            ,
            s if s.matches("failureReason") /* FailureReason com.amazonaws.ec2#VpcEndpointAssociation$FailureReason */ =>  {
                let var_6 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_failure_reason(var_6);
            }
            ,
            s if s.matches("failureCode") /* FailureCode com.amazonaws.ec2#VpcEndpointAssociation$FailureCode */ =>  {
                let var_7 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_failure_code(var_7);
            }
            ,
            s if s.matches("dnsEntry") /* DnsEntry com.amazonaws.ec2#VpcEndpointAssociation$DnsEntry */ =>  {
                let var_8 =
                    Some(
                        crate::protocol_serde::shape_dns_entry::de_dns_entry(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_dns_entry(var_8);
            }
            ,
            s if s.matches("privateDnsEntry") /* PrivateDnsEntry com.amazonaws.ec2#VpcEndpointAssociation$PrivateDnsEntry */ =>  {
                let var_9 =
                    Some(
                        crate::protocol_serde::shape_dns_entry::de_dns_entry(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_private_dns_entry(var_9);
            }
            ,
            s if s.matches("associatedResourceArn") /* AssociatedResourceArn com.amazonaws.ec2#VpcEndpointAssociation$AssociatedResourceArn */ =>  {
                let var_10 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_associated_resource_arn(var_10);
            }
            ,
            s if s.matches("resourceConfigurationGroupArn") /* ResourceConfigurationGroupArn com.amazonaws.ec2#VpcEndpointAssociation$ResourceConfigurationGroupArn */ =>  {
                let var_11 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_resource_configuration_group_arn(var_11);
            }
            ,
            s if s.matches("tagSet") /* Tags com.amazonaws.ec2#VpcEndpointAssociation$Tags */ =>  {
                let var_12 =
                    Some(
                        crate::protocol_serde::shape_tag_list::de_tag_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_tags(var_12);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

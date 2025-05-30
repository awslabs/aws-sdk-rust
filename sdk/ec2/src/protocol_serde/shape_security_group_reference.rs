// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_security_group_reference(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::SecurityGroupReference, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::SecurityGroupReference::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("groupId") /* GroupId com.amazonaws.ec2#SecurityGroupReference$GroupId */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_group_id(var_1);
            }
            ,
            s if s.matches("referencingVpcId") /* ReferencingVpcId com.amazonaws.ec2#SecurityGroupReference$ReferencingVpcId */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_referencing_vpc_id(var_2);
            }
            ,
            s if s.matches("vpcPeeringConnectionId") /* VpcPeeringConnectionId com.amazonaws.ec2#SecurityGroupReference$VpcPeeringConnectionId */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_vpc_peering_connection_id(var_3);
            }
            ,
            s if s.matches("transitGatewayId") /* TransitGatewayId com.amazonaws.ec2#SecurityGroupReference$TransitGatewayId */ =>  {
                let var_4 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_transit_gateway_id(var_4);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

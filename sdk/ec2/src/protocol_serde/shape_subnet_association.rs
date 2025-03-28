// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_subnet_association(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::SubnetAssociation, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::SubnetAssociation::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("subnetId") /* SubnetId com.amazonaws.ec2#SubnetAssociation$SubnetId */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_subnet_id(var_1);
            }
            ,
            s if s.matches("state") /* State com.amazonaws.ec2#SubnetAssociation$State */ =>  {
                let var_2 =
                    Some(
                        Result::<crate::types::TransitGatewayMulitcastDomainAssociationState, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::TransitGatewayMulitcastDomainAssociationState::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_state(var_2);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

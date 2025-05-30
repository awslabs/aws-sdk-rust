// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_subnet_ip_prefixes(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::SubnetIpPrefixes, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::SubnetIpPrefixes::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("subnetId") /* SubnetId com.amazonaws.ec2#SubnetIpPrefixes$SubnetId */ =>  {
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
            s if s.matches("ipPrefixSet") /* IpPrefixes com.amazonaws.ec2#SubnetIpPrefixes$IpPrefixes */ =>  {
                let var_2 =
                    Some(
                        crate::protocol_serde::shape_value_string_list::de_value_string_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_ip_prefixes(var_2);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

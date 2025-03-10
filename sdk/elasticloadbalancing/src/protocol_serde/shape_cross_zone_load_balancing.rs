// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(unused_mut)]
pub fn ser_cross_zone_load_balancing(
    mut writer: ::aws_smithy_query::QueryValueWriter,
    input: &crate::types::CrossZoneLoadBalancing,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("Enabled");
    {
        scope_1.boolean(input.enabled);
    }
    Ok(())
}

#[allow(clippy::needless_question_mark)]
pub fn de_cross_zone_load_balancing(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::CrossZoneLoadBalancing, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::CrossZoneLoadBalancing::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("Enabled") /* Enabled com.amazonaws.elasticloadbalancing#CrossZoneLoadBalancing$Enabled */ =>  {
                let var_2 =
                    Some(
                         {
                            <bool as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (boolean: `com.amazonaws.elasticloadbalancing#CrossZoneLoadBalancingEnabled`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_enabled(var_2);
            }
            ,
            _ => {}
        }
    }
    Ok(crate::serde_util::cross_zone_load_balancing_correct_errors(builder).build())
}

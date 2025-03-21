// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(unused_mut)]
pub fn ser_minimum_load_balancer_capacity(
    mut writer: ::aws_smithy_query::QueryValueWriter,
    input: &crate::types::MinimumLoadBalancerCapacity,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("CapacityUnits");
    if let Some(var_2) = &input.capacity_units {
        scope_1.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_2).into()),
        );
    }
    Ok(())
}

#[allow(clippy::needless_question_mark)]
pub fn de_minimum_load_balancer_capacity(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::MinimumLoadBalancerCapacity, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::MinimumLoadBalancerCapacity::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("CapacityUnits") /* CapacityUnits com.amazonaws.elasticloadbalancingv2#MinimumLoadBalancerCapacity$CapacityUnits */ =>  {
                let var_3 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.elasticloadbalancingv2#CapacityUnits`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_capacity_units(var_3);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

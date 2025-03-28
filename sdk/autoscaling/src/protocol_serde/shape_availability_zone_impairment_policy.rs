// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(unused_mut)]
pub fn ser_availability_zone_impairment_policy(
    mut writer: ::aws_smithy_query::QueryValueWriter,
    input: &crate::types::AvailabilityZoneImpairmentPolicy,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("ZonalShiftEnabled");
    if let Some(var_2) = &input.zonal_shift_enabled {
        scope_1.boolean(*var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("ImpairedZoneHealthCheckBehavior");
    if let Some(var_4) = &input.impaired_zone_health_check_behavior {
        scope_3.string(var_4.as_str());
    }
    Ok(())
}

#[allow(clippy::needless_question_mark)]
pub fn de_availability_zone_impairment_policy(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::AvailabilityZoneImpairmentPolicy, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::AvailabilityZoneImpairmentPolicy::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("ZonalShiftEnabled") /* ZonalShiftEnabled com.amazonaws.autoscaling#AvailabilityZoneImpairmentPolicy$ZonalShiftEnabled */ =>  {
                let var_5 =
                    Some(
                         {
                            <bool as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (boolean: `com.amazonaws.autoscaling#ZonalShiftEnabled`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_zonal_shift_enabled(var_5);
            }
            ,
            s if s.matches("ImpairedZoneHealthCheckBehavior") /* ImpairedZoneHealthCheckBehavior com.amazonaws.autoscaling#AvailabilityZoneImpairmentPolicy$ImpairedZoneHealthCheckBehavior */ =>  {
                let var_6 =
                    Some(
                        Result::<crate::types::ImpairedZoneHealthCheckBehavior, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::ImpairedZoneHealthCheckBehavior::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_impaired_zone_health_check_behavior(var_6);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

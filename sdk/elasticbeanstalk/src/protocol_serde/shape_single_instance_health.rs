// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_single_instance_health(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::SingleInstanceHealth, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::SingleInstanceHealth::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("InstanceId") /* InstanceId com.amazonaws.elasticbeanstalk#SingleInstanceHealth$InstanceId */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_instance_id(var_1);
            }
            ,
            s if s.matches("HealthStatus") /* HealthStatus com.amazonaws.elasticbeanstalk#SingleInstanceHealth$HealthStatus */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_health_status(var_2);
            }
            ,
            s if s.matches("Color") /* Color com.amazonaws.elasticbeanstalk#SingleInstanceHealth$Color */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_color(var_3);
            }
            ,
            s if s.matches("Causes") /* Causes com.amazonaws.elasticbeanstalk#SingleInstanceHealth$Causes */ =>  {
                let var_4 =
                    Some(
                        crate::protocol_serde::shape_causes::de_causes(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_causes(var_4);
            }
            ,
            s if s.matches("LaunchedAt") /* LaunchedAt com.amazonaws.elasticbeanstalk#SingleInstanceHealth$LaunchedAt */ =>  {
                let var_5 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.elasticbeanstalk#LaunchedAt`)"))
                        ?
                    )
                ;
                builder = builder.set_launched_at(var_5);
            }
            ,
            s if s.matches("ApplicationMetrics") /* ApplicationMetrics com.amazonaws.elasticbeanstalk#SingleInstanceHealth$ApplicationMetrics */ =>  {
                let var_6 =
                    Some(
                        crate::protocol_serde::shape_application_metrics::de_application_metrics(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_application_metrics(var_6);
            }
            ,
            s if s.matches("System") /* System com.amazonaws.elasticbeanstalk#SingleInstanceHealth$System */ =>  {
                let var_7 =
                    Some(
                        crate::protocol_serde::shape_system_status::de_system_status(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_system(var_7);
            }
            ,
            s if s.matches("Deployment") /* Deployment com.amazonaws.elasticbeanstalk#SingleInstanceHealth$Deployment */ =>  {
                let var_8 =
                    Some(
                        crate::protocol_serde::shape_deployment::de_deployment(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_deployment(var_8);
            }
            ,
            s if s.matches("AvailabilityZone") /* AvailabilityZone com.amazonaws.elasticbeanstalk#SingleInstanceHealth$AvailabilityZone */ =>  {
                let var_9 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_availability_zone(var_9);
            }
            ,
            s if s.matches("InstanceType") /* InstanceType com.amazonaws.elasticbeanstalk#SingleInstanceHealth$InstanceType */ =>  {
                let var_10 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_instance_type(var_10);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_insight_rule_metric_datapoint(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::InsightRuleMetricDatapoint, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::InsightRuleMetricDatapoint::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("Timestamp") /* Timestamp com.amazonaws.cloudwatch#InsightRuleMetricDatapoint$Timestamp */ =>  {
                let var_1 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.cloudwatch#Timestamp`)"))
                        ?
                    )
                ;
                builder = builder.set_timestamp(var_1);
            }
            ,
            s if s.matches("UniqueContributors") /* UniqueContributors com.amazonaws.cloudwatch#InsightRuleMetricDatapoint$UniqueContributors */ =>  {
                let var_2 =
                    Some(
                         {
                            <f64 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (double: `com.amazonaws.cloudwatch#InsightRuleUnboundDouble`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_unique_contributors(var_2);
            }
            ,
            s if s.matches("MaxContributorValue") /* MaxContributorValue com.amazonaws.cloudwatch#InsightRuleMetricDatapoint$MaxContributorValue */ =>  {
                let var_3 =
                    Some(
                         {
                            <f64 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (double: `com.amazonaws.cloudwatch#InsightRuleUnboundDouble`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_max_contributor_value(var_3);
            }
            ,
            s if s.matches("SampleCount") /* SampleCount com.amazonaws.cloudwatch#InsightRuleMetricDatapoint$SampleCount */ =>  {
                let var_4 =
                    Some(
                         {
                            <f64 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (double: `com.amazonaws.cloudwatch#InsightRuleUnboundDouble`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_sample_count(var_4);
            }
            ,
            s if s.matches("Average") /* Average com.amazonaws.cloudwatch#InsightRuleMetricDatapoint$Average */ =>  {
                let var_5 =
                    Some(
                         {
                            <f64 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (double: `com.amazonaws.cloudwatch#InsightRuleUnboundDouble`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_average(var_5);
            }
            ,
            s if s.matches("Sum") /* Sum com.amazonaws.cloudwatch#InsightRuleMetricDatapoint$Sum */ =>  {
                let var_6 =
                    Some(
                         {
                            <f64 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (double: `com.amazonaws.cloudwatch#InsightRuleUnboundDouble`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_sum(var_6);
            }
            ,
            s if s.matches("Minimum") /* Minimum com.amazonaws.cloudwatch#InsightRuleMetricDatapoint$Minimum */ =>  {
                let var_7 =
                    Some(
                         {
                            <f64 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (double: `com.amazonaws.cloudwatch#InsightRuleUnboundDouble`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_minimum(var_7);
            }
            ,
            s if s.matches("Maximum") /* Maximum com.amazonaws.cloudwatch#InsightRuleMetricDatapoint$Maximum */ =>  {
                let var_8 =
                    Some(
                         {
                            <f64 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (double: `com.amazonaws.cloudwatch#InsightRuleUnboundDouble`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_maximum(var_8);
            }
            ,
            _ => {}
        }
    }
    Ok(crate::serde_util::insight_rule_metric_datapoint_correct_errors(builder).build())
}

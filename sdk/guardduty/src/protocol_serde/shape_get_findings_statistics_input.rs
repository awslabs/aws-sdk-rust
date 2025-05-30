// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_get_findings_statistics_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::get_findings_statistics::GetFindingsStatisticsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.finding_criteria {
        #[allow(unused_mut)]
        let mut object_2 = object.key("findingCriteria").start_object();
        crate::protocol_serde::shape_finding_criteria::ser_finding_criteria(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.finding_statistic_types {
        let mut array_4 = object.key("findingStatisticTypes").start_array();
        for item_5 in var_3 {
            {
                array_4.value().string(item_5.as_str());
            }
        }
        array_4.finish();
    }
    if let Some(var_6) = &input.group_by {
        object.key("groupBy").string(var_6.as_str());
    }
    if let Some(var_7) = &input.max_results {
        object.key("maxResults").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_7).into()),
        );
    }
    if let Some(var_8) = &input.order_by {
        object.key("orderBy").string(var_8.as_str());
    }
    Ok(())
}

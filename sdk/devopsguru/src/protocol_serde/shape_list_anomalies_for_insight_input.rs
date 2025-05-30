// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_list_anomalies_for_insight_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::list_anomalies_for_insight::ListAnomaliesForInsightInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.account_id {
        object.key("AccountId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.filters {
        #[allow(unused_mut)]
        let mut object_3 = object.key("Filters").start_object();
        crate::protocol_serde::shape_list_anomalies_for_insight_filters::ser_list_anomalies_for_insight_filters(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.max_results {
        object.key("MaxResults").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_4).into()),
        );
    }
    if let Some(var_5) = &input.next_token {
        object.key("NextToken").string(var_5.as_str());
    }
    if let Some(var_6) = &input.start_time_range {
        #[allow(unused_mut)]
        let mut object_7 = object.key("StartTimeRange").start_object();
        crate::protocol_serde::shape_start_time_range::ser_start_time_range(&mut object_7, var_6)?;
        object_7.finish();
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_get_trace_summaries_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::get_trace_summaries::GetTraceSummariesInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.end_time {
        object
            .key("EndTime")
            .date_time(var_1, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    if let Some(var_2) = &input.filter_expression {
        object.key("FilterExpression").string(var_2.as_str());
    }
    if let Some(var_3) = &input.next_token {
        object.key("NextToken").string(var_3.as_str());
    }
    if let Some(var_4) = &input.sampling {
        object.key("Sampling").boolean(*var_4);
    }
    if let Some(var_5) = &input.sampling_strategy {
        #[allow(unused_mut)]
        let mut object_6 = object.key("SamplingStrategy").start_object();
        crate::protocol_serde::shape_sampling_strategy::ser_sampling_strategy(&mut object_6, var_5)?;
        object_6.finish();
    }
    if let Some(var_7) = &input.start_time {
        object
            .key("StartTime")
            .date_time(var_7, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    if let Some(var_8) = &input.time_range_type {
        object.key("TimeRangeType").string(var_8.as_str());
    }
    Ok(())
}

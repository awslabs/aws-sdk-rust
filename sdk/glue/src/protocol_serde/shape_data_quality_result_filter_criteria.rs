// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_data_quality_result_filter_criteria(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::DataQualityResultFilterCriteria,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.data_source {
        #[allow(unused_mut)]
        let mut object_2 = object.key("DataSource").start_object();
        crate::protocol_serde::shape_data_source::ser_data_source(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.job_name {
        object.key("JobName").string(var_3.as_str());
    }
    if let Some(var_4) = &input.job_run_id {
        object.key("JobRunId").string(var_4.as_str());
    }
    if let Some(var_5) = &input.started_after {
        object
            .key("StartedAfter")
            .date_time(var_5, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    if let Some(var_6) = &input.started_before {
        object
            .key("StartedBefore")
            .date_time(var_6, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    Ok(())
}

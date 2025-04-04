// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_get_anomaly_group_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::get_anomaly_group::GetAnomalyGroupInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.anomaly_detector_arn {
        object.key("AnomalyDetectorArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.anomaly_group_id {
        object.key("AnomalyGroupId").string(var_2.as_str());
    }
    Ok(())
}

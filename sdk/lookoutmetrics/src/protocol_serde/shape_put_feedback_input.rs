// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_put_feedback_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::put_feedback::PutFeedbackInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.anomaly_detector_arn {
        object.key("AnomalyDetectorArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.anomaly_group_time_series_feedback {
        #[allow(unused_mut)]
        let mut object_3 = object.key("AnomalyGroupTimeSeriesFeedback").start_object();
        crate::protocol_serde::shape_anomaly_group_time_series_feedback::ser_anomaly_group_time_series_feedback(&mut object_3, var_2)?;
        object_3.finish();
    }
    Ok(())
}

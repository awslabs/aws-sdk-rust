// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_event(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::Event,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.event_id {
        object.key("eventId").string(var_1.as_str());
    }
    {
        object.key("eventType").string(input.event_type.as_str());
    }
    if let Some(var_2) = &input.event_value {
        object.key("eventValue").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::Float((*var_2).into()),
        );
    }
    if let Some(var_3) = &input.item_id {
        object.key("itemId").string(var_3.as_str());
    }
    if let Some(var_4) = &input.properties {
        object.key("properties").string(var_4.as_str());
    }
    {
        object
            .key("sentAt")
            .date_time(&input.sent_at, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    if let Some(var_5) = &input.recommendation_id {
        object.key("recommendationId").string(var_5.as_str());
    }
    if let Some(var_6) = &input.impression {
        let mut array_7 = object.key("impression").start_array();
        for item_8 in var_6 {
            {
                array_7.value().string(item_8.as_str());
            }
        }
        array_7.finish();
    }
    if let Some(var_9) = &input.metric_attribution {
        #[allow(unused_mut)]
        let mut object_10 = object.key("metricAttribution").start_object();
        crate::protocol_serde::shape_metric_attribution::ser_metric_attribution(&mut object_10, var_9)?;
        object_10.finish();
    }
    Ok(())
}

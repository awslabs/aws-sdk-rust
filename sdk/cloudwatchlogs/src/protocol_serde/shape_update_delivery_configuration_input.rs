// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_delivery_configuration_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_delivery_configuration::UpdateDeliveryConfigurationInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.id {
        object.key("id").string(var_1.as_str());
    }
    if let Some(var_2) = &input.record_fields {
        let mut array_3 = object.key("recordFields").start_array();
        for item_4 in var_2 {
            {
                array_3.value().string(item_4.as_str());
            }
        }
        array_3.finish();
    }
    if let Some(var_5) = &input.field_delimiter {
        object.key("fieldDelimiter").string(var_5.as_str());
    }
    if let Some(var_6) = &input.s3_delivery_configuration {
        #[allow(unused_mut)]
        let mut object_7 = object.key("s3DeliveryConfiguration").start_object();
        crate::protocol_serde::shape_s3_delivery_configuration::ser_s3_delivery_configuration(&mut object_7, var_6)?;
        object_7.finish();
    }
    Ok(())
}

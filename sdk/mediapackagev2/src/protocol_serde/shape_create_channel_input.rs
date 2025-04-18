// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_channel_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_channel::CreateChannelInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.channel_name {
        object.key("ChannelName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.description {
        object.key("Description").string(var_2.as_str());
    }
    if let Some(var_3) = &input.input_switch_configuration {
        #[allow(unused_mut)]
        let mut object_4 = object.key("InputSwitchConfiguration").start_object();
        crate::protocol_serde::shape_input_switch_configuration::ser_input_switch_configuration(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.input_type {
        object.key("InputType").string(var_5.as_str());
    }
    if let Some(var_6) = &input.output_header_configuration {
        #[allow(unused_mut)]
        let mut object_7 = object.key("OutputHeaderConfiguration").start_object();
        crate::protocol_serde::shape_output_header_configuration::ser_output_header_configuration(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.tags {
        #[allow(unused_mut)]
        let mut object_9 = object.key("tags").start_object();
        for (key_10, value_11) in var_8 {
            {
                object_9.key(key_10.as_str()).string(value_11.as_str());
            }
        }
        object_9.finish();
    }
    Ok(())
}

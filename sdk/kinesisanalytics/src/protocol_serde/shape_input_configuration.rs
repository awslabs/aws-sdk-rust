// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_input_configuration(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::InputConfiguration,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("Id").string(input.id.as_str());
    }
    if let Some(var_1) = &input.input_starting_position_configuration {
        #[allow(unused_mut)]
        let mut object_2 = object.key("InputStartingPositionConfiguration").start_object();
        crate::protocol_serde::shape_input_starting_position_configuration::ser_input_starting_position_configuration(&mut object_2, var_1)?;
        object_2.finish();
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_attendee_capabilities_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_attendee_capabilities::UpdateAttendeeCapabilitiesInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.capabilities {
        #[allow(unused_mut)]
        let mut object_2 = object.key("Capabilities").start_object();
        crate::protocol_serde::shape_attendee_capabilities::ser_attendee_capabilities(&mut object_2, var_1)?;
        object_2.finish();
    }
    Ok(())
}

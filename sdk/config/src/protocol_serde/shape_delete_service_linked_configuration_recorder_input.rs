// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_delete_service_linked_configuration_recorder_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::delete_service_linked_configuration_recorder::DeleteServiceLinkedConfigurationRecorderInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.service_principal {
        object.key("ServicePrincipal").string(var_1.as_str());
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_protect_configuration_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_protect_configuration::UpdateProtectConfigurationInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.protect_configuration_id {
        object.key("ProtectConfigurationId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.deletion_protection_enabled {
        object.key("DeletionProtectionEnabled").boolean(*var_2);
    }
    Ok(())
}

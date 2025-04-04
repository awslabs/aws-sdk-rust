// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_automated_discovery_configuration_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_automated_discovery_configuration::UpdateAutomatedDiscoveryConfigurationInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.auto_enable_organization_members {
        object.key("autoEnableOrganizationMembers").string(var_1.as_str());
    }
    if let Some(var_2) = &input.status {
        object.key("status").string(var_2.as_str());
    }
    Ok(())
}

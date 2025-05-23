// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_put_selling_system_settings_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::put_selling_system_settings::PutSellingSystemSettingsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.catalog {
        object.key("Catalog").string(var_1.as_str());
    }
    if let Some(var_2) = &input.resource_snapshot_job_role_identifier {
        object.key("ResourceSnapshotJobRoleIdentifier").string(var_2.as_str());
    }
    Ok(())
}

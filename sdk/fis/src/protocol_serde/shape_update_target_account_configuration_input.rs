// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_target_account_configuration_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_target_account_configuration::UpdateTargetAccountConfigurationInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.description {
        object.key("description").string(var_1.as_str());
    }
    if let Some(var_2) = &input.role_arn {
        object.key("roleArn").string(var_2.as_str());
    }
    Ok(())
}

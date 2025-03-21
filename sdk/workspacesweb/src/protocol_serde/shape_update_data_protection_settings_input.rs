// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_data_protection_settings_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_data_protection_settings::UpdateDataProtectionSettingsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.client_token {
        object.key("clientToken").string(var_1.as_str());
    }
    if let Some(var_2) = &input.description {
        object.key("description").string(var_2.as_str());
    }
    if let Some(var_3) = &input.display_name {
        object.key("displayName").string(var_3.as_str());
    }
    if let Some(var_4) = &input.inline_redaction_configuration {
        #[allow(unused_mut)]
        let mut object_5 = object.key("inlineRedactionConfiguration").start_object();
        crate::protocol_serde::shape_inline_redaction_configuration::ser_inline_redaction_configuration(&mut object_5, var_4)?;
        object_5.finish();
    }
    Ok(())
}

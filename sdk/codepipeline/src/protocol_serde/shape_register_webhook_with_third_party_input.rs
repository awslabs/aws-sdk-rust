// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_register_webhook_with_third_party_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::register_webhook_with_third_party::RegisterWebhookWithThirdPartyInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.webhook_name {
        object.key("webhookName").string(var_1.as_str());
    }
    Ok(())
}

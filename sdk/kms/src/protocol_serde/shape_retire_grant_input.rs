// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_retire_grant_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::retire_grant::RetireGrantInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.grant_token {
        object.key("GrantToken").string(var_1.as_str());
    }
    if let Some(var_2) = &input.key_id {
        object.key("KeyId").string(var_2.as_str());
    }
    if let Some(var_3) = &input.grant_id {
        object.key("GrantId").string(var_3.as_str());
    }
    if let Some(var_4) = &input.dry_run {
        object.key("DryRun").boolean(*var_4);
    }
    Ok(())
}

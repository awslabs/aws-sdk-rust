// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_delete_slack_user_identity_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::delete_slack_user_identity::DeleteSlackUserIdentityInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.chat_configuration_arn {
        object.key("ChatConfigurationArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.slack_team_id {
        object.key("SlackTeamId").string(var_2.as_str());
    }
    if let Some(var_3) = &input.slack_user_id {
        object.key("SlackUserId").string(var_3.as_str());
    }
    Ok(())
}

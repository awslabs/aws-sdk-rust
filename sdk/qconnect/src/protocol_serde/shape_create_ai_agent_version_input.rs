// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_ai_agent_version_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_ai_agent_version::CreateAiAgentVersionInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.client_token {
        object.key("clientToken").string(var_1.as_str());
    }
    if let Some(var_2) = &input.modified_time {
        object
            .key("modifiedTime")
            .date_time(var_2, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    Ok(())
}

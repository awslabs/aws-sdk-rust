// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_get_violation_details_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::get_violation_details::GetViolationDetailsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.policy_id {
        object.key("PolicyId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.member_account {
        object.key("MemberAccount").string(var_2.as_str());
    }
    if let Some(var_3) = &input.resource_id {
        object.key("ResourceId").string(var_3.as_str());
    }
    if let Some(var_4) = &input.resource_type {
        object.key("ResourceType").string(var_4.as_str());
    }
    Ok(())
}

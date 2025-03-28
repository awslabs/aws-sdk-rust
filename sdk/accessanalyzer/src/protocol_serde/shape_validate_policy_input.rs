// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_validate_policy_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::validate_policy::ValidatePolicyInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.locale {
        object.key("locale").string(var_1.as_str());
    }
    if let Some(var_2) = &input.policy_document {
        object.key("policyDocument").string(var_2.as_str());
    }
    if let Some(var_3) = &input.policy_type {
        object.key("policyType").string(var_3.as_str());
    }
    if let Some(var_4) = &input.validate_policy_resource_type {
        object.key("validatePolicyResourceType").string(var_4.as_str());
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_describe_managed_rule_group_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::describe_managed_rule_group::DescribeManagedRuleGroupInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.vendor_name {
        object.key("VendorName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.name {
        object.key("Name").string(var_2.as_str());
    }
    if let Some(var_3) = &input.scope {
        object.key("Scope").string(var_3.as_str());
    }
    if let Some(var_4) = &input.version_name {
        object.key("VersionName").string(var_4.as_str());
    }
    Ok(())
}

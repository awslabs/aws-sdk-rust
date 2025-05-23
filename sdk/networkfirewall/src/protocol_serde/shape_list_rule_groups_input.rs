// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_list_rule_groups_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::list_rule_groups::ListRuleGroupsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.next_token {
        object.key("NextToken").string(var_1.as_str());
    }
    if let Some(var_2) = &input.max_results {
        object.key("MaxResults").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_2).into()),
        );
    }
    if let Some(var_3) = &input.scope {
        object.key("Scope").string(var_3.as_str());
    }
    if let Some(var_4) = &input.managed_type {
        object.key("ManagedType").string(var_4.as_str());
    }
    if let Some(var_5) = &input.r#type {
        object.key("Type").string(var_5.as_str());
    }
    Ok(())
}

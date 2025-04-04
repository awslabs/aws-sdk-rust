// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_execute_budget_action_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::execute_budget_action::ExecuteBudgetActionInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.account_id {
        object.key("AccountId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.budget_name {
        object.key("BudgetName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.action_id {
        object.key("ActionId").string(var_3.as_str());
    }
    if let Some(var_4) = &input.execution_type {
        object.key("ExecutionType").string(var_4.as_str());
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_list_application_operations_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::list_application_operations::ListApplicationOperationsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.application_name {
        object.key("ApplicationName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.limit {
        object.key("Limit").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_2).into()),
        );
    }
    if let Some(var_3) = &input.next_token {
        object.key("NextToken").string(var_3.as_str());
    }
    if let Some(var_4) = &input.operation {
        object.key("Operation").string(var_4.as_str());
    }
    if let Some(var_5) = &input.operation_status {
        object.key("OperationStatus").string(var_5.as_str());
    }
    Ok(())
}

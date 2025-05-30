// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_describe_maintenance_window_execution_task_invocations_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::describe_maintenance_window_execution_task_invocations::DescribeMaintenanceWindowExecutionTaskInvocationsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.window_execution_id {
        object.key("WindowExecutionId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.task_id {
        object.key("TaskId").string(var_2.as_str());
    }
    if let Some(var_3) = &input.filters {
        let mut array_4 = object.key("Filters").start_array();
        for item_5 in var_3 {
            {
                #[allow(unused_mut)]
                let mut object_6 = array_4.value().start_object();
                crate::protocol_serde::shape_maintenance_window_filter::ser_maintenance_window_filter(&mut object_6, item_5)?;
                object_6.finish();
            }
        }
        array_4.finish();
    }
    if let Some(var_7) = &input.max_results {
        object.key("MaxResults").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_7).into()),
        );
    }
    if let Some(var_8) = &input.next_token {
        object.key("NextToken").string(var_8.as_str());
    }
    Ok(())
}

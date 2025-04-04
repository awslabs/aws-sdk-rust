// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_batch_get_deployment_groups_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::batch_get_deployment_groups::BatchGetDeploymentGroupsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.application_name {
        object.key("applicationName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.deployment_group_names {
        let mut array_3 = object.key("deploymentGroupNames").start_array();
        for item_4 in var_2 {
            {
                array_3.value().string(item_4.as_str());
            }
        }
        array_3.finish();
    }
    Ok(())
}

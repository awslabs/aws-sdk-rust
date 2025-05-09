// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_get_workload_deployment_pattern_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::get_workload_deployment_pattern::GetWorkloadDeploymentPatternInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.deployment_pattern_name {
        object.key("deploymentPatternName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.workload_name {
        object.key("workloadName").string(var_2.as_str());
    }
    Ok(())
}

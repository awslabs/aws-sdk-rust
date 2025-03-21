// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_configuration_definition_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ConfigurationDefinitionInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("Type").string(input.r#type.as_str());
    }
    {
        #[allow(unused_mut)]
        let mut object_1 = object.key("Parameters").start_object();
        for (key_2, value_3) in &input.parameters {
            {
                object_1.key(key_2.as_str()).string(value_3.as_str());
            }
        }
        object_1.finish();
    }
    if let Some(var_4) = &input.type_version {
        object.key("TypeVersion").string(var_4.as_str());
    }
    if let Some(var_5) = &input.local_deployment_execution_role_name {
        object.key("LocalDeploymentExecutionRoleName").string(var_5.as_str());
    }
    if let Some(var_6) = &input.local_deployment_administration_role_arn {
        object.key("LocalDeploymentAdministrationRoleArn").string(var_6.as_str());
    }
    Ok(())
}

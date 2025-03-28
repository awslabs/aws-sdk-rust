// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_application_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_application::UpdateApplicationInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.application_name {
        object.key("ApplicationName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.current_application_version_id {
        object.key("CurrentApplicationVersionId").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_2).into()),
        );
    }
    if let Some(var_3) = &input.application_configuration_update {
        #[allow(unused_mut)]
        let mut object_4 = object.key("ApplicationConfigurationUpdate").start_object();
        crate::protocol_serde::shape_application_configuration_update::ser_application_configuration_update(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.service_execution_role_update {
        object.key("ServiceExecutionRoleUpdate").string(var_5.as_str());
    }
    if let Some(var_6) = &input.run_configuration_update {
        #[allow(unused_mut)]
        let mut object_7 = object.key("RunConfigurationUpdate").start_object();
        crate::protocol_serde::shape_run_configuration_update::ser_run_configuration_update(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.cloud_watch_logging_option_updates {
        let mut array_9 = object.key("CloudWatchLoggingOptionUpdates").start_array();
        for item_10 in var_8 {
            {
                #[allow(unused_mut)]
                let mut object_11 = array_9.value().start_object();
                crate::protocol_serde::shape_cloud_watch_logging_option_update::ser_cloud_watch_logging_option_update(&mut object_11, item_10)?;
                object_11.finish();
            }
        }
        array_9.finish();
    }
    if let Some(var_12) = &input.conditional_token {
        object.key("ConditionalToken").string(var_12.as_str());
    }
    if let Some(var_13) = &input.runtime_environment_update {
        object.key("RuntimeEnvironmentUpdate").string(var_13.as_str());
    }
    Ok(())
}

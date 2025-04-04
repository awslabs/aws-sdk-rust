// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_robot_application_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_robot_application::UpdateRobotApplicationInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.application {
        object.key("application").string(var_1.as_str());
    }
    if let Some(var_2) = &input.current_revision_id {
        object.key("currentRevisionId").string(var_2.as_str());
    }
    if let Some(var_3) = &input.environment {
        #[allow(unused_mut)]
        let mut object_4 = object.key("environment").start_object();
        crate::protocol_serde::shape_environment::ser_environment(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.robot_software_suite {
        #[allow(unused_mut)]
        let mut object_6 = object.key("robotSoftwareSuite").start_object();
        crate::protocol_serde::shape_robot_software_suite::ser_robot_software_suite(&mut object_6, var_5)?;
        object_6.finish();
    }
    if let Some(var_7) = &input.sources {
        let mut array_8 = object.key("sources").start_array();
        for item_9 in var_7 {
            {
                #[allow(unused_mut)]
                let mut object_10 = array_8.value().start_object();
                crate::protocol_serde::shape_source_config::ser_source_config(&mut object_10, item_9)?;
                object_10.finish();
            }
        }
        array_8.finish();
    }
    Ok(())
}

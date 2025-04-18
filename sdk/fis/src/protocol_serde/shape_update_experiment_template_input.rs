// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_experiment_template_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_experiment_template::UpdateExperimentTemplateInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.actions {
        #[allow(unused_mut)]
        let mut object_2 = object.key("actions").start_object();
        for (key_3, value_4) in var_1 {
            {
                #[allow(unused_mut)]
                let mut object_5 = object_2.key(key_3.as_str()).start_object();
                crate::protocol_serde::shape_update_experiment_template_action_input_item::ser_update_experiment_template_action_input_item(
                    &mut object_5,
                    value_4,
                )?;
                object_5.finish();
            }
        }
        object_2.finish();
    }
    if let Some(var_6) = &input.description {
        object.key("description").string(var_6.as_str());
    }
    if let Some(var_7) = &input.experiment_options {
        #[allow(unused_mut)]
        let mut object_8 = object.key("experimentOptions").start_object();
        crate::protocol_serde::shape_update_experiment_template_experiment_options_input::ser_update_experiment_template_experiment_options_input(
            &mut object_8,
            var_7,
        )?;
        object_8.finish();
    }
    if let Some(var_9) = &input.experiment_report_configuration {
        #[allow(unused_mut)]
        let mut object_10 = object.key("experimentReportConfiguration").start_object();
        crate::protocol_serde::shape_update_experiment_template_report_configuration_input::ser_update_experiment_template_report_configuration_input(&mut object_10, var_9)?;
        object_10.finish();
    }
    if let Some(var_11) = &input.log_configuration {
        #[allow(unused_mut)]
        let mut object_12 = object.key("logConfiguration").start_object();
        crate::protocol_serde::shape_update_experiment_template_log_configuration_input::ser_update_experiment_template_log_configuration_input(
            &mut object_12,
            var_11,
        )?;
        object_12.finish();
    }
    if let Some(var_13) = &input.role_arn {
        object.key("roleArn").string(var_13.as_str());
    }
    if let Some(var_14) = &input.stop_conditions {
        let mut array_15 = object.key("stopConditions").start_array();
        for item_16 in var_14 {
            {
                #[allow(unused_mut)]
                let mut object_17 = array_15.value().start_object();
                crate::protocol_serde::shape_update_experiment_template_stop_condition_input::ser_update_experiment_template_stop_condition_input(
                    &mut object_17,
                    item_16,
                )?;
                object_17.finish();
            }
        }
        array_15.finish();
    }
    if let Some(var_18) = &input.targets {
        #[allow(unused_mut)]
        let mut object_19 = object.key("targets").start_object();
        for (key_20, value_21) in var_18 {
            {
                #[allow(unused_mut)]
                let mut object_22 = object_19.key(key_20.as_str()).start_object();
                crate::protocol_serde::shape_update_experiment_template_target_input::ser_update_experiment_template_target_input(
                    &mut object_22,
                    value_21,
                )?;
                object_22.finish();
            }
        }
        object_19.finish();
    }
    Ok(())
}

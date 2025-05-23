// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_run_task_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::run_task::RunTaskInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.capacity_provider_strategy {
        let mut array_2 = object.key("capacityProviderStrategy").start_array();
        for item_3 in var_1 {
            {
                #[allow(unused_mut)]
                let mut object_4 = array_2.value().start_object();
                crate::protocol_serde::shape_capacity_provider_strategy_item::ser_capacity_provider_strategy_item(&mut object_4, item_3)?;
                object_4.finish();
            }
        }
        array_2.finish();
    }
    if let Some(var_5) = &input.cluster {
        object.key("cluster").string(var_5.as_str());
    }
    if let Some(var_6) = &input.count {
        object.key("count").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_6).into()),
        );
    }
    if let Some(var_7) = &input.enable_ecs_managed_tags {
        object.key("enableECSManagedTags").boolean(*var_7);
    }
    if let Some(var_8) = &input.enable_execute_command {
        object.key("enableExecuteCommand").boolean(*var_8);
    }
    if let Some(var_9) = &input.group {
        object.key("group").string(var_9.as_str());
    }
    if let Some(var_10) = &input.launch_type {
        object.key("launchType").string(var_10.as_str());
    }
    if let Some(var_11) = &input.network_configuration {
        #[allow(unused_mut)]
        let mut object_12 = object.key("networkConfiguration").start_object();
        crate::protocol_serde::shape_network_configuration::ser_network_configuration(&mut object_12, var_11)?;
        object_12.finish();
    }
    if let Some(var_13) = &input.overrides {
        #[allow(unused_mut)]
        let mut object_14 = object.key("overrides").start_object();
        crate::protocol_serde::shape_task_override::ser_task_override(&mut object_14, var_13)?;
        object_14.finish();
    }
    if let Some(var_15) = &input.placement_constraints {
        let mut array_16 = object.key("placementConstraints").start_array();
        for item_17 in var_15 {
            {
                #[allow(unused_mut)]
                let mut object_18 = array_16.value().start_object();
                crate::protocol_serde::shape_placement_constraint::ser_placement_constraint(&mut object_18, item_17)?;
                object_18.finish();
            }
        }
        array_16.finish();
    }
    if let Some(var_19) = &input.placement_strategy {
        let mut array_20 = object.key("placementStrategy").start_array();
        for item_21 in var_19 {
            {
                #[allow(unused_mut)]
                let mut object_22 = array_20.value().start_object();
                crate::protocol_serde::shape_placement_strategy::ser_placement_strategy(&mut object_22, item_21)?;
                object_22.finish();
            }
        }
        array_20.finish();
    }
    if let Some(var_23) = &input.platform_version {
        object.key("platformVersion").string(var_23.as_str());
    }
    if let Some(var_24) = &input.propagate_tags {
        object.key("propagateTags").string(var_24.as_str());
    }
    if let Some(var_25) = &input.reference_id {
        object.key("referenceId").string(var_25.as_str());
    }
    if let Some(var_26) = &input.started_by {
        object.key("startedBy").string(var_26.as_str());
    }
    if let Some(var_27) = &input.tags {
        let mut array_28 = object.key("tags").start_array();
        for item_29 in var_27 {
            {
                #[allow(unused_mut)]
                let mut object_30 = array_28.value().start_object();
                crate::protocol_serde::shape_tag::ser_tag(&mut object_30, item_29)?;
                object_30.finish();
            }
        }
        array_28.finish();
    }
    if let Some(var_31) = &input.task_definition {
        object.key("taskDefinition").string(var_31.as_str());
    }
    if let Some(var_32) = &input.client_token {
        object.key("clientToken").string(var_32.as_str());
    }
    if let Some(var_33) = &input.volume_configurations {
        let mut array_34 = object.key("volumeConfigurations").start_array();
        for item_35 in var_33 {
            {
                #[allow(unused_mut)]
                let mut object_36 = array_34.value().start_object();
                crate::protocol_serde::shape_task_volume_configuration::ser_task_volume_configuration(&mut object_36, item_35)?;
                object_36.finish();
            }
        }
        array_34.finish();
    }
    Ok(())
}

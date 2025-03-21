// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_service_level_objective_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_service_level_objective::CreateServiceLevelObjectiveInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.burn_rate_configurations {
        let mut array_2 = object.key("BurnRateConfigurations").start_array();
        for item_3 in var_1 {
            {
                #[allow(unused_mut)]
                let mut object_4 = array_2.value().start_object();
                crate::protocol_serde::shape_burn_rate_configuration::ser_burn_rate_configuration(&mut object_4, item_3)?;
                object_4.finish();
            }
        }
        array_2.finish();
    }
    if let Some(var_5) = &input.description {
        object.key("Description").string(var_5.as_str());
    }
    if let Some(var_6) = &input.goal {
        #[allow(unused_mut)]
        let mut object_7 = object.key("Goal").start_object();
        crate::protocol_serde::shape_goal::ser_goal(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.name {
        object.key("Name").string(var_8.as_str());
    }
    if let Some(var_9) = &input.request_based_sli_config {
        #[allow(unused_mut)]
        let mut object_10 = object.key("RequestBasedSliConfig").start_object();
        crate::protocol_serde::shape_request_based_service_level_indicator_config::ser_request_based_service_level_indicator_config(
            &mut object_10,
            var_9,
        )?;
        object_10.finish();
    }
    if let Some(var_11) = &input.sli_config {
        #[allow(unused_mut)]
        let mut object_12 = object.key("SliConfig").start_object();
        crate::protocol_serde::shape_service_level_indicator_config::ser_service_level_indicator_config(&mut object_12, var_11)?;
        object_12.finish();
    }
    if let Some(var_13) = &input.tags {
        let mut array_14 = object.key("Tags").start_array();
        for item_15 in var_13 {
            {
                #[allow(unused_mut)]
                let mut object_16 = array_14.value().start_object();
                crate::protocol_serde::shape_tag::ser_tag(&mut object_16, item_15)?;
                object_16.finish();
            }
        }
        array_14.finish();
    }
    Ok(())
}

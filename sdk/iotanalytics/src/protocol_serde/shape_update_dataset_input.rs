// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_dataset_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_dataset::UpdateDatasetInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.actions {
        let mut array_2 = object.key("actions").start_array();
        for item_3 in var_1 {
            {
                #[allow(unused_mut)]
                let mut object_4 = array_2.value().start_object();
                crate::protocol_serde::shape_dataset_action::ser_dataset_action(&mut object_4, item_3)?;
                object_4.finish();
            }
        }
        array_2.finish();
    }
    if let Some(var_5) = &input.content_delivery_rules {
        let mut array_6 = object.key("contentDeliveryRules").start_array();
        for item_7 in var_5 {
            {
                #[allow(unused_mut)]
                let mut object_8 = array_6.value().start_object();
                crate::protocol_serde::shape_dataset_content_delivery_rule::ser_dataset_content_delivery_rule(&mut object_8, item_7)?;
                object_8.finish();
            }
        }
        array_6.finish();
    }
    if let Some(var_9) = &input.late_data_rules {
        let mut array_10 = object.key("lateDataRules").start_array();
        for item_11 in var_9 {
            {
                #[allow(unused_mut)]
                let mut object_12 = array_10.value().start_object();
                crate::protocol_serde::shape_late_data_rule::ser_late_data_rule(&mut object_12, item_11)?;
                object_12.finish();
            }
        }
        array_10.finish();
    }
    if let Some(var_13) = &input.retention_period {
        #[allow(unused_mut)]
        let mut object_14 = object.key("retentionPeriod").start_object();
        crate::protocol_serde::shape_retention_period::ser_retention_period(&mut object_14, var_13)?;
        object_14.finish();
    }
    if let Some(var_15) = &input.triggers {
        let mut array_16 = object.key("triggers").start_array();
        for item_17 in var_15 {
            {
                #[allow(unused_mut)]
                let mut object_18 = array_16.value().start_object();
                crate::protocol_serde::shape_dataset_trigger::ser_dataset_trigger(&mut object_18, item_17)?;
                object_18.finish();
            }
        }
        array_16.finish();
    }
    if let Some(var_19) = &input.versioning_configuration {
        #[allow(unused_mut)]
        let mut object_20 = object.key("versioningConfiguration").start_object();
        crate::protocol_serde::shape_versioning_configuration::ser_versioning_configuration(&mut object_20, var_19)?;
        object_20.finish();
    }
    Ok(())
}

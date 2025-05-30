// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_component_data(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::CreateComponentData,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("name").string(input.name.as_str());
    }
    if let Some(var_1) = &input.source_id {
        object.key("sourceId").string(var_1.as_str());
    }
    {
        object.key("componentType").string(input.component_type.as_str());
    }
    {
        #[allow(unused_mut)]
        let mut object_2 = object.key("properties").start_object();
        for (key_3, value_4) in &input.properties {
            {
                #[allow(unused_mut)]
                let mut object_5 = object_2.key(key_3.as_str()).start_object();
                crate::protocol_serde::shape_component_property::ser_component_property(&mut object_5, value_4)?;
                object_5.finish();
            }
        }
        object_2.finish();
    }
    if let Some(var_6) = &input.children {
        let mut array_7 = object.key("children").start_array();
        for item_8 in var_6 {
            {
                #[allow(unused_mut)]
                let mut object_9 = array_7.value().start_object();
                crate::protocol_serde::shape_component_child::ser_component_child(&mut object_9, item_8)?;
                object_9.finish();
            }
        }
        array_7.finish();
    }
    {
        let mut array_10 = object.key("variants").start_array();
        for item_11 in &input.variants {
            {
                #[allow(unused_mut)]
                let mut object_12 = array_10.value().start_object();
                crate::protocol_serde::shape_component_variant::ser_component_variant(&mut object_12, item_11)?;
                object_12.finish();
            }
        }
        array_10.finish();
    }
    {
        #[allow(unused_mut)]
        let mut object_13 = object.key("overrides").start_object();
        for (key_14, value_15) in &input.overrides {
            {
                #[allow(unused_mut)]
                let mut object_16 = object_13.key(key_14.as_str()).start_object();
                for (key_17, value_18) in value_15 {
                    {
                        object_16.key(key_17.as_str()).string(value_18.as_str());
                    }
                }
                object_16.finish();
            }
        }
        object_13.finish();
    }
    {
        #[allow(unused_mut)]
        let mut object_19 = object.key("bindingProperties").start_object();
        for (key_20, value_21) in &input.binding_properties {
            {
                #[allow(unused_mut)]
                let mut object_22 = object_19.key(key_20.as_str()).start_object();
                crate::protocol_serde::shape_component_binding_properties_value::ser_component_binding_properties_value(&mut object_22, value_21)?;
                object_22.finish();
            }
        }
        object_19.finish();
    }
    if let Some(var_23) = &input.collection_properties {
        #[allow(unused_mut)]
        let mut object_24 = object.key("collectionProperties").start_object();
        for (key_25, value_26) in var_23 {
            {
                #[allow(unused_mut)]
                let mut object_27 = object_24.key(key_25.as_str()).start_object();
                crate::protocol_serde::shape_component_data_configuration::ser_component_data_configuration(&mut object_27, value_26)?;
                object_27.finish();
            }
        }
        object_24.finish();
    }
    if let Some(var_28) = &input.tags {
        #[allow(unused_mut)]
        let mut object_29 = object.key("tags").start_object();
        for (key_30, value_31) in var_28 {
            {
                object_29.key(key_30.as_str()).string(value_31.as_str());
            }
        }
        object_29.finish();
    }
    if let Some(var_32) = &input.events {
        #[allow(unused_mut)]
        let mut object_33 = object.key("events").start_object();
        for (key_34, value_35) in var_32 {
            {
                #[allow(unused_mut)]
                let mut object_36 = object_33.key(key_34.as_str()).start_object();
                crate::protocol_serde::shape_component_event::ser_component_event(&mut object_36, value_35)?;
                object_36.finish();
            }
        }
        object_33.finish();
    }
    if let Some(var_37) = &input.schema_version {
        object.key("schemaVersion").string(var_37.as_str());
    }
    Ok(())
}

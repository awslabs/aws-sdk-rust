// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_scene_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_scene::CreateSceneInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.capabilities {
        let mut array_2 = object.key("capabilities").start_array();
        for item_3 in var_1 {
            {
                array_2.value().string(item_3.as_str());
            }
        }
        array_2.finish();
    }
    if let Some(var_4) = &input.content_location {
        object.key("contentLocation").string(var_4.as_str());
    }
    if let Some(var_5) = &input.description {
        object.key("description").string(var_5.as_str());
    }
    if let Some(var_6) = &input.scene_id {
        object.key("sceneId").string(var_6.as_str());
    }
    if let Some(var_7) = &input.scene_metadata {
        #[allow(unused_mut)]
        let mut object_8 = object.key("sceneMetadata").start_object();
        for (key_9, value_10) in var_7 {
            {
                object_8.key(key_9.as_str()).string(value_10.as_str());
            }
        }
        object_8.finish();
    }
    if let Some(var_11) = &input.tags {
        #[allow(unused_mut)]
        let mut object_12 = object.key("tags").start_object();
        for (key_13, value_14) in var_11 {
            {
                object_12.key(key_13.as_str()).string(value_14.as_str());
            }
        }
        object_12.finish();
    }
    Ok(())
}

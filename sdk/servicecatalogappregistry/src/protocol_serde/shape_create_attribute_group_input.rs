// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_attribute_group_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_attribute_group::CreateAttributeGroupInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.attributes {
        object.key("attributes").string(var_1.as_str());
    }
    if let Some(var_2) = &input.client_token {
        object.key("clientToken").string(var_2.as_str());
    }
    if let Some(var_3) = &input.description {
        object.key("description").string(var_3.as_str());
    }
    if let Some(var_4) = &input.name {
        object.key("name").string(var_4.as_str());
    }
    if let Some(var_5) = &input.tags {
        #[allow(unused_mut)]
        let mut object_6 = object.key("tags").start_object();
        for (key_7, value_8) in var_5 {
            {
                object_6.key(key_7.as_str()).string(value_8.as_str());
            }
        }
        object_6.finish();
    }
    Ok(())
}

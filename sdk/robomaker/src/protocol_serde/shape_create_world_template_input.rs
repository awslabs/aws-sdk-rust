// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_world_template_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_world_template::CreateWorldTemplateInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.client_request_token {
        object.key("clientRequestToken").string(var_1.as_str());
    }
    if let Some(var_2) = &input.name {
        object.key("name").string(var_2.as_str());
    }
    if let Some(var_3) = &input.tags {
        #[allow(unused_mut)]
        let mut object_4 = object.key("tags").start_object();
        for (key_5, value_6) in var_3 {
            {
                object_4.key(key_5.as_str()).string(value_6.as_str());
            }
        }
        object_4.finish();
    }
    if let Some(var_7) = &input.template_body {
        object.key("templateBody").string(var_7.as_str());
    }
    if let Some(var_8) = &input.template_location {
        #[allow(unused_mut)]
        let mut object_9 = object.key("templateLocation").start_object();
        crate::protocol_serde::shape_template_location::ser_template_location(&mut object_9, var_8)?;
        object_9.finish();
    }
    Ok(())
}

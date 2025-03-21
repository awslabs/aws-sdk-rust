// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_put_parameter_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::put_parameter::PutParameterInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.name {
        object.key("Name").string(var_1.as_str());
    }
    if let Some(var_2) = &input.description {
        object.key("Description").string(var_2.as_str());
    }
    if let Some(var_3) = &input.value {
        object.key("Value").string(var_3.as_str());
    }
    if let Some(var_4) = &input.r#type {
        object.key("Type").string(var_4.as_str());
    }
    if let Some(var_5) = &input.key_id {
        object.key("KeyId").string(var_5.as_str());
    }
    if let Some(var_6) = &input.overwrite {
        object.key("Overwrite").boolean(*var_6);
    }
    if let Some(var_7) = &input.allowed_pattern {
        object.key("AllowedPattern").string(var_7.as_str());
    }
    if let Some(var_8) = &input.tags {
        let mut array_9 = object.key("Tags").start_array();
        for item_10 in var_8 {
            {
                #[allow(unused_mut)]
                let mut object_11 = array_9.value().start_object();
                crate::protocol_serde::shape_tag::ser_tag(&mut object_11, item_10)?;
                object_11.finish();
            }
        }
        array_9.finish();
    }
    if let Some(var_12) = &input.tier {
        object.key("Tier").string(var_12.as_str());
    }
    if let Some(var_13) = &input.policies {
        object.key("Policies").string(var_13.as_str());
    }
    if let Some(var_14) = &input.data_type {
        object.key("DataType").string(var_14.as_str());
    }
    Ok(())
}

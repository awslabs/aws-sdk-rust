// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_fleet_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_fleet::CreateFleetInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.configuration {
        #[allow(unused_mut)]
        let mut object_2 = object.key("configuration").start_object();
        crate::protocol_serde::shape_fleet_configuration::ser_fleet_configuration(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.description {
        object.key("description").string(var_3.as_str());
    }
    if let Some(var_4) = &input.display_name {
        object.key("displayName").string(var_4.as_str());
    }
    if let Some(var_5) = &input.max_worker_count {
        object.key("maxWorkerCount").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_5).into()),
        );
    }
    if let Some(var_6) = &input.min_worker_count {
        object.key("minWorkerCount").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_6).into()),
        );
    }
    if let Some(var_7) = &input.role_arn {
        object.key("roleArn").string(var_7.as_str());
    }
    if let Some(var_8) = &input.tags {
        #[allow(unused_mut)]
        let mut object_9 = object.key("tags").start_object();
        for (key_10, value_11) in var_8 {
            {
                object_9.key(key_10.as_str()).string(value_11.as_str());
            }
        }
        object_9.finish();
    }
    Ok(())
}

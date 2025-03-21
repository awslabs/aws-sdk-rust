// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_environment_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_environment::CreateEnvironmentInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.data_bundles {
        let mut array_2 = object.key("dataBundles").start_array();
        for item_3 in var_1 {
            {
                array_2.value().string(item_3.as_str());
            }
        }
        array_2.finish();
    }
    if let Some(var_4) = &input.description {
        object.key("description").string(var_4.as_str());
    }
    if let Some(var_5) = &input.federation_mode {
        object.key("federationMode").string(var_5.as_str());
    }
    if let Some(var_6) = &input.federation_parameters {
        #[allow(unused_mut)]
        let mut object_7 = object.key("federationParameters").start_object();
        crate::protocol_serde::shape_federation_parameters::ser_federation_parameters(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.kms_key_id {
        object.key("kmsKeyId").string(var_8.as_str());
    }
    if let Some(var_9) = &input.name {
        object.key("name").string(var_9.as_str());
    }
    if let Some(var_10) = &input.superuser_parameters {
        #[allow(unused_mut)]
        let mut object_11 = object.key("superuserParameters").start_object();
        crate::protocol_serde::shape_superuser_parameters::ser_superuser_parameters(&mut object_11, var_10)?;
        object_11.finish();
    }
    if let Some(var_12) = &input.tags {
        #[allow(unused_mut)]
        let mut object_13 = object.key("tags").start_object();
        for (key_14, value_15) in var_12 {
            {
                object_13.key(key_14.as_str()).string(value_15.as_str());
            }
        }
        object_13.finish();
    }
    Ok(())
}

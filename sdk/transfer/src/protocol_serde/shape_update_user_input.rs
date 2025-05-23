// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_user_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_user::UpdateUserInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.home_directory {
        object.key("HomeDirectory").string(var_1.as_str());
    }
    if let Some(var_2) = &input.home_directory_type {
        object.key("HomeDirectoryType").string(var_2.as_str());
    }
    if let Some(var_3) = &input.home_directory_mappings {
        let mut array_4 = object.key("HomeDirectoryMappings").start_array();
        for item_5 in var_3 {
            {
                #[allow(unused_mut)]
                let mut object_6 = array_4.value().start_object();
                crate::protocol_serde::shape_home_directory_map_entry::ser_home_directory_map_entry(&mut object_6, item_5)?;
                object_6.finish();
            }
        }
        array_4.finish();
    }
    if let Some(var_7) = &input.policy {
        object.key("Policy").string(var_7.as_str());
    }
    if let Some(var_8) = &input.posix_profile {
        #[allow(unused_mut)]
        let mut object_9 = object.key("PosixProfile").start_object();
        crate::protocol_serde::shape_posix_profile::ser_posix_profile(&mut object_9, var_8)?;
        object_9.finish();
    }
    if let Some(var_10) = &input.role {
        object.key("Role").string(var_10.as_str());
    }
    if let Some(var_11) = &input.server_id {
        object.key("ServerId").string(var_11.as_str());
    }
    if let Some(var_12) = &input.user_name {
        object.key("UserName").string(var_12.as_str());
    }
    Ok(())
}

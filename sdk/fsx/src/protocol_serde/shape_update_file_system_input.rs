// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_file_system_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_file_system::UpdateFileSystemInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.file_system_id {
        object.key("FileSystemId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.client_request_token {
        object.key("ClientRequestToken").string(var_2.as_str());
    }
    if let Some(var_3) = &input.storage_capacity {
        object.key("StorageCapacity").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_3).into()),
        );
    }
    if let Some(var_4) = &input.windows_configuration {
        #[allow(unused_mut)]
        let mut object_5 = object.key("WindowsConfiguration").start_object();
        crate::protocol_serde::shape_update_file_system_windows_configuration::ser_update_file_system_windows_configuration(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.lustre_configuration {
        #[allow(unused_mut)]
        let mut object_7 = object.key("LustreConfiguration").start_object();
        crate::protocol_serde::shape_update_file_system_lustre_configuration::ser_update_file_system_lustre_configuration(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.ontap_configuration {
        #[allow(unused_mut)]
        let mut object_9 = object.key("OntapConfiguration").start_object();
        crate::protocol_serde::shape_update_file_system_ontap_configuration::ser_update_file_system_ontap_configuration(&mut object_9, var_8)?;
        object_9.finish();
    }
    if let Some(var_10) = &input.open_zfs_configuration {
        #[allow(unused_mut)]
        let mut object_11 = object.key("OpenZFSConfiguration").start_object();
        crate::protocol_serde::shape_update_file_system_open_zfs_configuration::ser_update_file_system_open_zfs_configuration(
            &mut object_11,
            var_10,
        )?;
        object_11.finish();
    }
    if let Some(var_12) = &input.storage_type {
        object.key("StorageType").string(var_12.as_str());
    }
    if let Some(var_13) = &input.file_system_type_version {
        object.key("FileSystemTypeVersion").string(var_13.as_str());
    }
    Ok(())
}

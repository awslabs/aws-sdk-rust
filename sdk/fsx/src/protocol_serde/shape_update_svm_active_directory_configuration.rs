// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_svm_active_directory_configuration(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::UpdateSvmActiveDirectoryConfiguration,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.self_managed_active_directory_configuration {
        #[allow(unused_mut)]
        let mut object_2 = object.key("SelfManagedActiveDirectoryConfiguration").start_object();
        crate::protocol_serde::shape_self_managed_active_directory_configuration_updates::ser_self_managed_active_directory_configuration_updates(
            &mut object_2,
            var_1,
        )?;
        object_2.finish();
    }
    if let Some(var_3) = &input.net_bios_name {
        object.key("NetBiosName").string(var_3.as_str());
    }
    Ok(())
}

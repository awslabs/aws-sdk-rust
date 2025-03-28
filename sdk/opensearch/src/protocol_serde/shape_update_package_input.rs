// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_package_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_package::UpdatePackageInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.commit_message {
        object.key("CommitMessage").string(var_1.as_str());
    }
    if let Some(var_2) = &input.package_configuration {
        #[allow(unused_mut)]
        let mut object_3 = object.key("PackageConfiguration").start_object();
        crate::protocol_serde::shape_package_configuration::ser_package_configuration(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.package_description {
        object.key("PackageDescription").string(var_4.as_str());
    }
    if let Some(var_5) = &input.package_encryption_options {
        #[allow(unused_mut)]
        let mut object_6 = object.key("PackageEncryptionOptions").start_object();
        crate::protocol_serde::shape_package_encryption_options::ser_package_encryption_options(&mut object_6, var_5)?;
        object_6.finish();
    }
    if let Some(var_7) = &input.package_id {
        object.key("PackageID").string(var_7.as_str());
    }
    if let Some(var_8) = &input.package_source {
        #[allow(unused_mut)]
        let mut object_9 = object.key("PackageSource").start_object();
        crate::protocol_serde::shape_package_source::ser_package_source(&mut object_9, var_8)?;
        object_9.finish();
    }
    Ok(())
}

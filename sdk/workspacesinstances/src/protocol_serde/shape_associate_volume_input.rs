// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_associate_volume_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::associate_volume::AssociateVolumeInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.workspace_instance_id {
        object.key("WorkspaceInstanceId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.volume_id {
        object.key("VolumeId").string(var_2.as_str());
    }
    if let Some(var_3) = &input.device {
        object.key("Device").string(var_3.as_str());
    }
    Ok(())
}

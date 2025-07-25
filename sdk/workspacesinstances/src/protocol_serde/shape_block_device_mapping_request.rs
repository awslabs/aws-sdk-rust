// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_block_device_mapping_request(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::BlockDeviceMappingRequest,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.device_name {
        object.key("DeviceName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.ebs {
        #[allow(unused_mut)]
        let mut object_3 = object.key("Ebs").start_object();
        crate::protocol_serde::shape_ebs_block_device::ser_ebs_block_device(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.no_device {
        object.key("NoDevice").string(var_4.as_str());
    }
    if let Some(var_5) = &input.virtual_name {
        object.key("VirtualName").string(var_5.as_str());
    }
    Ok(())
}

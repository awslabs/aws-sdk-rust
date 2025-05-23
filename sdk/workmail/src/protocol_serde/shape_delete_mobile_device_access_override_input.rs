// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_delete_mobile_device_access_override_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::delete_mobile_device_access_override::DeleteMobileDeviceAccessOverrideInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.organization_id {
        object.key("OrganizationId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.user_id {
        object.key("UserId").string(var_2.as_str());
    }
    if let Some(var_3) = &input.device_id {
        object.key("DeviceId").string(var_3.as_str());
    }
    Ok(())
}

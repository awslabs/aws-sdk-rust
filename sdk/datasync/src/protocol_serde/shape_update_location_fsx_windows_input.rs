// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_location_fsx_windows_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_location_fsx_windows::UpdateLocationFsxWindowsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.location_arn {
        object.key("LocationArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.subdirectory {
        object.key("Subdirectory").string(var_2.as_str());
    }
    if let Some(var_3) = &input.domain {
        object.key("Domain").string(var_3.as_str());
    }
    if let Some(var_4) = &input.user {
        object.key("User").string(var_4.as_str());
    }
    if let Some(var_5) = &input.password {
        object.key("Password").string(var_5.as_str());
    }
    Ok(())
}

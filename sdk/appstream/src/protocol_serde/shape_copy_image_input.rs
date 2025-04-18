// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_copy_image_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::copy_image::CopyImageInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.source_image_name {
        object.key("SourceImageName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.destination_image_name {
        object.key("DestinationImageName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.destination_region {
        object.key("DestinationRegion").string(var_3.as_str());
    }
    if let Some(var_4) = &input.destination_image_description {
        object.key("DestinationImageDescription").string(var_4.as_str());
    }
    Ok(())
}

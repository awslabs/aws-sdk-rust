// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_json_classifier_request(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::UpdateJsonClassifierRequest,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("Name").string(input.name.as_str());
    }
    if let Some(var_1) = &input.json_path {
        object.key("JsonPath").string(var_1.as_str());
    }
    Ok(())
}

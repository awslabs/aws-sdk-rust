// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_condition(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::Condition,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("Type").string(input.r#type.as_str());
    }
    {
        object.key("Key").string(input.key.as_str());
    }
    {
        object.key("Value").string(input.value.as_str());
    }
    Ok(())
}

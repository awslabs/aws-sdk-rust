// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_type_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_type::CreateTypeInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.definition {
        object.key("definition").string(var_1.as_str());
    }
    if let Some(var_2) = &input.format {
        object.key("format").string(var_2.as_str());
    }
    Ok(())
}

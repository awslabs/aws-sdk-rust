// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_http_namespace_change(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::HttpNamespaceChange,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("Description").string(input.description.as_str());
    }
    Ok(())
}

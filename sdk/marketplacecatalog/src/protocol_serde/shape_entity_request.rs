// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_entity_request(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::EntityRequest,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("Catalog").string(input.catalog.as_str());
    }
    {
        object.key("EntityId").string(input.entity_id.as_str());
    }
    Ok(())
}

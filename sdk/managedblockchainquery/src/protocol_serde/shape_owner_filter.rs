// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_owner_filter(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::OwnerFilter,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("address").string(input.address.as_str());
    }
    Ok(())
}

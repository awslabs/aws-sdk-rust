// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_component_version(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ComponentVersion,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("componentType").string(input.component_type.as_str());
    }
    {
        let mut array_1 = object.key("versions").start_array();
        for item_2 in &input.versions {
            {
                array_1.value().string(item_2.as_str());
            }
        }
        array_1.finish();
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_batch_get_link_attributes(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::BatchGetLinkAttributes,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.typed_link_specifier {
        #[allow(unused_mut)]
        let mut object_2 = object.key("TypedLinkSpecifier").start_object();
        crate::protocol_serde::shape_typed_link_specifier::ser_typed_link_specifier(&mut object_2, var_1)?;
        object_2.finish();
    }
    {
        let mut array_3 = object.key("AttributeNames").start_array();
        for item_4 in &input.attribute_names {
            {
                array_3.value().string(item_4.as_str());
            }
        }
        array_3.finish();
    }
    Ok(())
}

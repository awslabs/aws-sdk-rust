// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_model_metadata_search_expression(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ModelMetadataSearchExpression,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.filters {
        let mut array_2 = object.key("Filters").start_array();
        for item_3 in var_1 {
            {
                #[allow(unused_mut)]
                let mut object_4 = array_2.value().start_object();
                crate::protocol_serde::shape_model_metadata_filter::ser_model_metadata_filter(&mut object_4, item_3)?;
                object_4.finish();
            }
        }
        array_2.finish();
    }
    Ok(())
}

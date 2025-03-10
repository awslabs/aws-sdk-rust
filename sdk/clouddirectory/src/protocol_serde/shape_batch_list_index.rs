// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_batch_list_index(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::BatchListIndex,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.ranges_on_indexed_values {
        let mut array_2 = object.key("RangesOnIndexedValues").start_array();
        for item_3 in var_1 {
            {
                #[allow(unused_mut)]
                let mut object_4 = array_2.value().start_object();
                crate::protocol_serde::shape_object_attribute_range::ser_object_attribute_range(&mut object_4, item_3)?;
                object_4.finish();
            }
        }
        array_2.finish();
    }
    if let Some(var_5) = &input.index_reference {
        #[allow(unused_mut)]
        let mut object_6 = object.key("IndexReference").start_object();
        crate::protocol_serde::shape_object_reference::ser_object_reference(&mut object_6, var_5)?;
        object_6.finish();
    }
    if let Some(var_7) = &input.max_results {
        object.key("MaxResults").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_7).into()),
        );
    }
    if let Some(var_8) = &input.next_token {
        object.key("NextToken").string(var_8.as_str());
    }
    Ok(())
}

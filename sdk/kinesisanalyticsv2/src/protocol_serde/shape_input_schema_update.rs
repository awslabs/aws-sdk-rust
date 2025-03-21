// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_input_schema_update(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::InputSchemaUpdate,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.record_format_update {
        #[allow(unused_mut)]
        let mut object_2 = object.key("RecordFormatUpdate").start_object();
        crate::protocol_serde::shape_record_format::ser_record_format(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.record_encoding_update {
        object.key("RecordEncodingUpdate").string(var_3.as_str());
    }
    if let Some(var_4) = &input.record_column_updates {
        let mut array_5 = object.key("RecordColumnUpdates").start_array();
        for item_6 in var_4 {
            {
                #[allow(unused_mut)]
                let mut object_7 = array_5.value().start_object();
                crate::protocol_serde::shape_record_column::ser_record_column(&mut object_7, item_6)?;
                object_7.finish();
            }
        }
        array_5.finish();
    }
    Ok(())
}

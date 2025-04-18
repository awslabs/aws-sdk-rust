// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_table_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_table::CreateTableInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.database_name {
        object.key("DatabaseName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.table_name {
        object.key("TableName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.retention_properties {
        #[allow(unused_mut)]
        let mut object_4 = object.key("RetentionProperties").start_object();
        crate::protocol_serde::shape_retention_properties::ser_retention_properties(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.tags {
        let mut array_6 = object.key("Tags").start_array();
        for item_7 in var_5 {
            {
                #[allow(unused_mut)]
                let mut object_8 = array_6.value().start_object();
                crate::protocol_serde::shape_tag::ser_tag(&mut object_8, item_7)?;
                object_8.finish();
            }
        }
        array_6.finish();
    }
    if let Some(var_9) = &input.magnetic_store_write_properties {
        #[allow(unused_mut)]
        let mut object_10 = object.key("MagneticStoreWriteProperties").start_object();
        crate::protocol_serde::shape_magnetic_store_write_properties::ser_magnetic_store_write_properties(&mut object_10, var_9)?;
        object_10.finish();
    }
    if let Some(var_11) = &input.schema {
        #[allow(unused_mut)]
        let mut object_12 = object.key("Schema").start_object();
        crate::protocol_serde::shape_schema::ser_schema(&mut object_12, var_11)?;
        object_12.finish();
    }
    Ok(())
}

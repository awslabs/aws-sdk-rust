// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_partition_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_partition::CreatePartitionInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.catalog_id {
        object.key("CatalogId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.database_name {
        object.key("DatabaseName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.table_name {
        object.key("TableName").string(var_3.as_str());
    }
    if let Some(var_4) = &input.partition_input {
        #[allow(unused_mut)]
        let mut object_5 = object.key("PartitionInput").start_object();
        crate::protocol_serde::shape_partition_input::ser_partition_input(&mut object_5, var_4)?;
        object_5.finish();
    }
    Ok(())
}

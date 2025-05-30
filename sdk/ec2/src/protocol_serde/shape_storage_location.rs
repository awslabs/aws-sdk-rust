// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(unused_mut)]
pub fn ser_storage_location(
    mut writer: ::aws_smithy_query::QueryValueWriter,
    input: &crate::types::StorageLocation,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("Bucket");
    if let Some(var_2) = &input.bucket {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("Key");
    if let Some(var_4) = &input.key {
        scope_3.string(var_4);
    }
    Ok(())
}

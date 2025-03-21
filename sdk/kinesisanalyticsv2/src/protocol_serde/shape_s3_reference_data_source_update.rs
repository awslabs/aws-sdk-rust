// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_s3_reference_data_source_update(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::S3ReferenceDataSourceUpdate,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.bucket_arn_update {
        object.key("BucketARNUpdate").string(var_1.as_str());
    }
    if let Some(var_2) = &input.file_key_update {
        object.key("FileKeyUpdate").string(var_2.as_str());
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_store_image_task_input_input_input(
    input: &crate::operation::create_store_image_task::CreateStoreImageTaskInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "CreateStoreImageTask", "2016-11-15");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("ImageId");
    if let Some(var_2) = &input.image_id {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("Bucket");
    if let Some(var_4) = &input.bucket {
        scope_3.string(var_4);
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("S3ObjectTag");
    if let Some(var_6) = &input.s3_object_tags {
        if !var_6.is_empty() {
            let mut list_8 = scope_5.start_list(true, Some("item"));
            for item_7 in var_6 {
                #[allow(unused_mut)]
                let mut entry_9 = list_8.entry();
                crate::protocol_serde::shape_s3_object_tag::ser_s3_object_tag(entry_9, item_7)?;
            }
            list_8.finish();
        }
    }
    #[allow(unused_mut)]
    let mut scope_10 = writer.prefix("DryRun");
    if let Some(var_11) = &input.dry_run {
        scope_10.boolean(*var_11);
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_custom_db_engine_version_input_input_input(
    input: &crate::operation::create_custom_db_engine_version::CreateCustomDbEngineVersionInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "CreateCustomDBEngineVersion", "2014-10-31");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("Engine");
    if let Some(var_2) = &input.engine {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("EngineVersion");
    if let Some(var_4) = &input.engine_version {
        scope_3.string(var_4);
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("DatabaseInstallationFilesS3BucketName");
    if let Some(var_6) = &input.database_installation_files_s3_bucket_name {
        scope_5.string(var_6);
    }
    #[allow(unused_mut)]
    let mut scope_7 = writer.prefix("DatabaseInstallationFilesS3Prefix");
    if let Some(var_8) = &input.database_installation_files_s3_prefix {
        scope_7.string(var_8);
    }
    #[allow(unused_mut)]
    let mut scope_9 = writer.prefix("ImageId");
    if let Some(var_10) = &input.image_id {
        scope_9.string(var_10);
    }
    #[allow(unused_mut)]
    let mut scope_11 = writer.prefix("KMSKeyId");
    if let Some(var_12) = &input.kms_key_id {
        scope_11.string(var_12);
    }
    #[allow(unused_mut)]
    let mut scope_13 = writer.prefix("Description");
    if let Some(var_14) = &input.description {
        scope_13.string(var_14);
    }
    #[allow(unused_mut)]
    let mut scope_15 = writer.prefix("Manifest");
    if let Some(var_16) = &input.manifest {
        scope_15.string(var_16);
    }
    #[allow(unused_mut)]
    let mut scope_17 = writer.prefix("Tags");
    if let Some(var_18) = &input.tags {
        let mut list_20 = scope_17.start_list(false, Some("Tag"));
        for item_19 in var_18 {
            #[allow(unused_mut)]
            let mut entry_21 = list_20.entry();
            crate::protocol_serde::shape_tag::ser_tag(entry_21, item_19)?;
        }
        list_20.finish();
    }
    #[allow(unused_mut)]
    let mut scope_22 = writer.prefix("SourceCustomDbEngineVersionIdentifier");
    if let Some(var_23) = &input.source_custom_db_engine_version_identifier {
        scope_22.string(var_23);
    }
    #[allow(unused_mut)]
    let mut scope_24 = writer.prefix("UseAwsProvidedLatestImage");
    if let Some(var_25) = &input.use_aws_provided_latest_image {
        scope_24.boolean(*var_25);
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

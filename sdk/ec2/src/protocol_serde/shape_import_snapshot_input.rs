// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_import_snapshot_input_input_input(
    input: &crate::operation::import_snapshot::ImportSnapshotInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "ImportSnapshot", "2016-11-15");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("ClientData");
    if let Some(var_2) = &input.client_data {
        crate::protocol_serde::shape_client_data::ser_client_data(scope_1, var_2)?;
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("ClientToken");
    if let Some(var_4) = &input.client_token {
        scope_3.string(var_4);
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("Description");
    if let Some(var_6) = &input.description {
        scope_5.string(var_6);
    }
    #[allow(unused_mut)]
    let mut scope_7 = writer.prefix("DiskContainer");
    if let Some(var_8) = &input.disk_container {
        crate::protocol_serde::shape_snapshot_disk_container::ser_snapshot_disk_container(scope_7, var_8)?;
    }
    #[allow(unused_mut)]
    let mut scope_9 = writer.prefix("DryRun");
    if let Some(var_10) = &input.dry_run {
        scope_9.boolean(*var_10);
    }
    #[allow(unused_mut)]
    let mut scope_11 = writer.prefix("Encrypted");
    if let Some(var_12) = &input.encrypted {
        scope_11.boolean(*var_12);
    }
    #[allow(unused_mut)]
    let mut scope_13 = writer.prefix("KmsKeyId");
    if let Some(var_14) = &input.kms_key_id {
        scope_13.string(var_14);
    }
    #[allow(unused_mut)]
    let mut scope_15 = writer.prefix("RoleName");
    if let Some(var_16) = &input.role_name {
        scope_15.string(var_16);
    }
    #[allow(unused_mut)]
    let mut scope_17 = writer.prefix("TagSpecification");
    if let Some(var_18) = &input.tag_specifications {
        if !var_18.is_empty() {
            let mut list_20 = scope_17.start_list(true, Some("item"));
            for item_19 in var_18 {
                #[allow(unused_mut)]
                let mut entry_21 = list_20.entry();
                crate::protocol_serde::shape_tag_specification::ser_tag_specification(entry_21, item_19)?;
            }
            list_20.finish();
        }
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

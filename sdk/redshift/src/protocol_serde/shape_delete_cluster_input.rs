// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_delete_cluster_input_input_input(
    input: &crate::operation::delete_cluster::DeleteClusterInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "DeleteCluster", "2012-12-01");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("ClusterIdentifier");
    if let Some(var_2) = &input.cluster_identifier {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("SkipFinalClusterSnapshot");
    if let Some(var_4) = &input.skip_final_cluster_snapshot {
        scope_3.boolean(*var_4);
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("FinalClusterSnapshotIdentifier");
    if let Some(var_6) = &input.final_cluster_snapshot_identifier {
        scope_5.string(var_6);
    }
    #[allow(unused_mut)]
    let mut scope_7 = writer.prefix("FinalClusterSnapshotRetentionPeriod");
    if let Some(var_8) = &input.final_cluster_snapshot_retention_period {
        scope_7.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_8).into()),
        );
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

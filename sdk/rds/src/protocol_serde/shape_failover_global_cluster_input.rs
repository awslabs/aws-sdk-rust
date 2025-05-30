// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_failover_global_cluster_input_input_input(
    input: &crate::operation::failover_global_cluster::FailoverGlobalClusterInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "FailoverGlobalCluster", "2014-10-31");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("GlobalClusterIdentifier");
    if let Some(var_2) = &input.global_cluster_identifier {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("TargetDbClusterIdentifier");
    if let Some(var_4) = &input.target_db_cluster_identifier {
        scope_3.string(var_4);
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("AllowDataLoss");
    if let Some(var_6) = &input.allow_data_loss {
        scope_5.boolean(*var_6);
    }
    #[allow(unused_mut)]
    let mut scope_7 = writer.prefix("Switchover");
    if let Some(var_8) = &input.switchover {
        scope_7.boolean(*var_8);
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

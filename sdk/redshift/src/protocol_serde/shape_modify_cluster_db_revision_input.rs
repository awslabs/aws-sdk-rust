// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_modify_cluster_db_revision_input_input_input(
    input: &crate::operation::modify_cluster_db_revision::ModifyClusterDbRevisionInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "ModifyClusterDbRevision", "2012-12-01");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("ClusterIdentifier");
    if let Some(var_2) = &input.cluster_identifier {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("RevisionTarget");
    if let Some(var_4) = &input.revision_target {
        scope_3.string(var_4);
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn de_db_cluster_roles(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<::std::vec::Vec<crate::types::DbClusterRole>, ::aws_smithy_xml::decode::XmlDecodeError> {
    let mut out = std::vec::Vec::new();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("DBClusterRole") /* member com.amazonaws.docdb#DBClusterRoles$member */ =>  {
                out.push(
                    crate::protocol_serde::shape_db_cluster_role::de_db_cluster_role(&mut tag)
                    ?
                );
            }
            ,
            _ => {}
        }
    }
    Ok(out)
}

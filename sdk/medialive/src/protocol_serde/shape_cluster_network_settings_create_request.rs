// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_cluster_network_settings_create_request(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ClusterNetworkSettingsCreateRequest,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.default_route {
        object.key("defaultRoute").string(var_1.as_str());
    }
    if let Some(var_2) = &input.interface_mappings {
        let mut array_3 = object.key("interfaceMappings").start_array();
        for item_4 in var_2 {
            {
                #[allow(unused_mut)]
                let mut object_5 = array_3.value().start_object();
                crate::protocol_serde::shape_interface_mapping_create_request::ser_interface_mapping_create_request(&mut object_5, item_4)?;
                object_5.finish();
            }
        }
        array_3.finish();
    }
    Ok(())
}

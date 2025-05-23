// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_deregister_transit_gateway_multicast_group_members_input_input_input(
    input: &crate::operation::deregister_transit_gateway_multicast_group_members::DeregisterTransitGatewayMulticastGroupMembersInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "DeregisterTransitGatewayMulticastGroupMembers", "2016-11-15");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("TransitGatewayMulticastDomainId");
    if let Some(var_2) = &input.transit_gateway_multicast_domain_id {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("GroupIpAddress");
    if let Some(var_4) = &input.group_ip_address {
        scope_3.string(var_4);
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("NetworkInterfaceIds");
    if let Some(var_6) = &input.network_interface_ids {
        if !var_6.is_empty() {
            let mut list_8 = scope_5.start_list(true, Some("item"));
            for item_7 in var_6 {
                #[allow(unused_mut)]
                let mut entry_9 = list_8.entry();
                entry_9.string(item_7);
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

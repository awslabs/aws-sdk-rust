// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(unused_mut)]
pub fn ser_scheduled_instances_network_interface(
    mut writer: ::aws_smithy_query::QueryValueWriter,
    input: &crate::types::ScheduledInstancesNetworkInterface,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("AssociatePublicIpAddress");
    if let Some(var_2) = &input.associate_public_ip_address {
        scope_1.boolean(*var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("DeleteOnTermination");
    if let Some(var_4) = &input.delete_on_termination {
        scope_3.boolean(*var_4);
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("Description");
    if let Some(var_6) = &input.description {
        scope_5.string(var_6);
    }
    #[allow(unused_mut)]
    let mut scope_7 = writer.prefix("DeviceIndex");
    if let Some(var_8) = &input.device_index {
        scope_7.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_8).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_9 = writer.prefix("Group");
    if let Some(var_10) = &input.groups {
        if !var_10.is_empty() {
            let mut list_12 = scope_9.start_list(true, Some("SecurityGroupId"));
            for item_11 in var_10 {
                #[allow(unused_mut)]
                let mut entry_13 = list_12.entry();
                entry_13.string(item_11);
            }
            list_12.finish();
        }
    }
    #[allow(unused_mut)]
    let mut scope_14 = writer.prefix("Ipv6AddressCount");
    if let Some(var_15) = &input.ipv6_address_count {
        scope_14.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_15).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_16 = writer.prefix("Ipv6Address");
    if let Some(var_17) = &input.ipv6_addresses {
        if !var_17.is_empty() {
            let mut list_19 = scope_16.start_list(true, Some("Ipv6Address"));
            for item_18 in var_17 {
                #[allow(unused_mut)]
                let mut entry_20 = list_19.entry();
                crate::protocol_serde::shape_scheduled_instances_ipv6_address::ser_scheduled_instances_ipv6_address(entry_20, item_18)?;
            }
            list_19.finish();
        }
    }
    #[allow(unused_mut)]
    let mut scope_21 = writer.prefix("NetworkInterfaceId");
    if let Some(var_22) = &input.network_interface_id {
        scope_21.string(var_22);
    }
    #[allow(unused_mut)]
    let mut scope_23 = writer.prefix("PrivateIpAddress");
    if let Some(var_24) = &input.private_ip_address {
        scope_23.string(var_24);
    }
    #[allow(unused_mut)]
    let mut scope_25 = writer.prefix("PrivateIpAddressConfig");
    if let Some(var_26) = &input.private_ip_address_configs {
        if !var_26.is_empty() {
            let mut list_28 = scope_25.start_list(true, Some("PrivateIpAddressConfigSet"));
            for item_27 in var_26 {
                #[allow(unused_mut)]
                let mut entry_29 = list_28.entry();
                crate::protocol_serde::shape_scheduled_instances_private_ip_address_config::ser_scheduled_instances_private_ip_address_config(
                    entry_29, item_27,
                )?;
            }
            list_28.finish();
        }
    }
    #[allow(unused_mut)]
    let mut scope_30 = writer.prefix("SecondaryPrivateIpAddressCount");
    if let Some(var_31) = &input.secondary_private_ip_address_count {
        scope_30.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_31).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_32 = writer.prefix("SubnetId");
    if let Some(var_33) = &input.subnet_id {
        scope_32.string(var_33);
    }
    Ok(())
}

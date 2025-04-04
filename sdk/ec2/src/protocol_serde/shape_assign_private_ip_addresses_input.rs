// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_assign_private_ip_addresses_input_input_input(
    input: &crate::operation::assign_private_ip_addresses::AssignPrivateIpAddressesInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "AssignPrivateIpAddresses", "2016-11-15");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("Ipv4Prefix");
    if let Some(var_2) = &input.ipv4_prefixes {
        if !var_2.is_empty() {
            let mut list_4 = scope_1.start_list(true, Some("item"));
            for item_3 in var_2 {
                #[allow(unused_mut)]
                let mut entry_5 = list_4.entry();
                entry_5.string(item_3);
            }
            list_4.finish();
        }
    }
    #[allow(unused_mut)]
    let mut scope_6 = writer.prefix("Ipv4PrefixCount");
    if let Some(var_7) = &input.ipv4_prefix_count {
        scope_6.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_7).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_8 = writer.prefix("NetworkInterfaceId");
    if let Some(var_9) = &input.network_interface_id {
        scope_8.string(var_9);
    }
    #[allow(unused_mut)]
    let mut scope_10 = writer.prefix("PrivateIpAddress");
    if let Some(var_11) = &input.private_ip_addresses {
        if !var_11.is_empty() {
            let mut list_13 = scope_10.start_list(true, Some("PrivateIpAddress"));
            for item_12 in var_11 {
                #[allow(unused_mut)]
                let mut entry_14 = list_13.entry();
                entry_14.string(item_12);
            }
            list_13.finish();
        }
    }
    #[allow(unused_mut)]
    let mut scope_15 = writer.prefix("SecondaryPrivateIpAddressCount");
    if let Some(var_16) = &input.secondary_private_ip_address_count {
        scope_15.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_16).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_17 = writer.prefix("AllowReassignment");
    if let Some(var_18) = &input.allow_reassignment {
        scope_17.boolean(*var_18);
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

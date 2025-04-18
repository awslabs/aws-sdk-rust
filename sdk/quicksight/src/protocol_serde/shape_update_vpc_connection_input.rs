// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_vpc_connection_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_vpc_connection::UpdateVpcConnectionInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.dns_resolvers {
        let mut array_2 = object.key("DnsResolvers").start_array();
        for item_3 in var_1 {
            {
                array_2.value().string(item_3.as_str());
            }
        }
        array_2.finish();
    }
    if let Some(var_4) = &input.name {
        object.key("Name").string(var_4.as_str());
    }
    if let Some(var_5) = &input.role_arn {
        object.key("RoleArn").string(var_5.as_str());
    }
    if let Some(var_6) = &input.security_group_ids {
        let mut array_7 = object.key("SecurityGroupIds").start_array();
        for item_8 in var_6 {
            {
                array_7.value().string(item_8.as_str());
            }
        }
        array_7.finish();
    }
    if let Some(var_9) = &input.subnet_ids {
        let mut array_10 = object.key("SubnetIds").start_array();
        for item_11 in var_9 {
            {
                array_10.value().string(item_11.as_str());
            }
        }
        array_10.finish();
    }
    Ok(())
}

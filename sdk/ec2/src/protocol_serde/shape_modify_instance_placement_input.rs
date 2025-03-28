// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_modify_instance_placement_input_input_input(
    input: &crate::operation::modify_instance_placement::ModifyInstancePlacementInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "ModifyInstancePlacement", "2016-11-15");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("GroupName");
    if let Some(var_2) = &input.group_name {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("PartitionNumber");
    if let Some(var_4) = &input.partition_number {
        scope_3.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_4).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("HostResourceGroupArn");
    if let Some(var_6) = &input.host_resource_group_arn {
        scope_5.string(var_6);
    }
    #[allow(unused_mut)]
    let mut scope_7 = writer.prefix("GroupId");
    if let Some(var_8) = &input.group_id {
        scope_7.string(var_8);
    }
    #[allow(unused_mut)]
    let mut scope_9 = writer.prefix("InstanceId");
    if let Some(var_10) = &input.instance_id {
        scope_9.string(var_10);
    }
    #[allow(unused_mut)]
    let mut scope_11 = writer.prefix("Tenancy");
    if let Some(var_12) = &input.tenancy {
        scope_11.string(var_12.as_str());
    }
    #[allow(unused_mut)]
    let mut scope_13 = writer.prefix("Affinity");
    if let Some(var_14) = &input.affinity {
        scope_13.string(var_14.as_str());
    }
    #[allow(unused_mut)]
    let mut scope_15 = writer.prefix("HostId");
    if let Some(var_16) = &input.host_id {
        scope_15.string(var_16);
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

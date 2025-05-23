// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(unused_mut)]
pub fn ser_launch_template_ebs_block_device_request(
    mut writer: ::aws_smithy_query::QueryValueWriter,
    input: &crate::types::LaunchTemplateEbsBlockDeviceRequest,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("Encrypted");
    if let Some(var_2) = &input.encrypted {
        scope_1.boolean(*var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("DeleteOnTermination");
    if let Some(var_4) = &input.delete_on_termination {
        scope_3.boolean(*var_4);
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("Iops");
    if let Some(var_6) = &input.iops {
        scope_5.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_6).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_7 = writer.prefix("KmsKeyId");
    if let Some(var_8) = &input.kms_key_id {
        scope_7.string(var_8);
    }
    #[allow(unused_mut)]
    let mut scope_9 = writer.prefix("SnapshotId");
    if let Some(var_10) = &input.snapshot_id {
        scope_9.string(var_10);
    }
    #[allow(unused_mut)]
    let mut scope_11 = writer.prefix("VolumeSize");
    if let Some(var_12) = &input.volume_size {
        scope_11.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_12).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_13 = writer.prefix("VolumeType");
    if let Some(var_14) = &input.volume_type {
        scope_13.string(var_14.as_str());
    }
    #[allow(unused_mut)]
    let mut scope_15 = writer.prefix("Throughput");
    if let Some(var_16) = &input.throughput {
        scope_15.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_16).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_17 = writer.prefix("VolumeInitializationRate");
    if let Some(var_18) = &input.volume_initialization_rate {
        scope_17.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_18).into()),
        );
    }
    Ok(())
}

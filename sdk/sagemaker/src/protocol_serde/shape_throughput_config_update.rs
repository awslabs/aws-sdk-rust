// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_throughput_config_update(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ThroughputConfigUpdate,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.throughput_mode {
        object.key("ThroughputMode").string(var_1.as_str());
    }
    if let Some(var_2) = &input.provisioned_read_capacity_units {
        object.key("ProvisionedReadCapacityUnits").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_2).into()),
        );
    }
    if let Some(var_3) = &input.provisioned_write_capacity_units {
        object.key("ProvisionedWriteCapacityUnits").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_3).into()),
        );
    }
    Ok(())
}

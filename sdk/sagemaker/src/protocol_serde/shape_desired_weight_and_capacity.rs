// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_desired_weight_and_capacity(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::DesiredWeightAndCapacity,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.variant_name {
        object.key("VariantName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.desired_weight {
        object.key("DesiredWeight").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::Float((*var_2).into()),
        );
    }
    if let Some(var_3) = &input.desired_instance_count {
        object.key("DesiredInstanceCount").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_3).into()),
        );
    }
    if let Some(var_4) = &input.serverless_update_config {
        #[allow(unused_mut)]
        let mut object_5 = object.key("ServerlessUpdateConfig").start_object();
        crate::protocol_serde::shape_production_variant_serverless_update_config::ser_production_variant_serverless_update_config(
            &mut object_5,
            var_4,
        )?;
        object_5.finish();
    }
    Ok(())
}

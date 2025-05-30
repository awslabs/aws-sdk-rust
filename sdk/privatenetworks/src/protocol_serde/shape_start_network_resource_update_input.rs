// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_start_network_resource_update_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::start_network_resource_update::StartNetworkResourceUpdateInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.commitment_configuration {
        #[allow(unused_mut)]
        let mut object_2 = object.key("commitmentConfiguration").start_object();
        crate::protocol_serde::shape_commitment_configuration::ser_commitment_configuration(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.network_resource_arn {
        object.key("networkResourceArn").string(var_3.as_str());
    }
    if let Some(var_4) = &input.return_reason {
        object.key("returnReason").string(var_4.as_str());
    }
    if let Some(var_5) = &input.shipping_address {
        #[allow(unused_mut)]
        let mut object_6 = object.key("shippingAddress").start_object();
        crate::protocol_serde::shape_address::ser_address(&mut object_6, var_5)?;
        object_6.finish();
    }
    if let Some(var_7) = &input.update_type {
        object.key("updateType").string(var_7.as_str());
    }
    Ok(())
}

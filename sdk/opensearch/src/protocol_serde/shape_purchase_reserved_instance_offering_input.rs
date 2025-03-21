// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_purchase_reserved_instance_offering_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::purchase_reserved_instance_offering::PurchaseReservedInstanceOfferingInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.instance_count {
        object.key("InstanceCount").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_1).into()),
        );
    }
    if let Some(var_2) = &input.reservation_name {
        object.key("ReservationName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.reserved_instance_offering_id {
        object.key("ReservedInstanceOfferingId").string(var_3.as_str());
    }
    Ok(())
}

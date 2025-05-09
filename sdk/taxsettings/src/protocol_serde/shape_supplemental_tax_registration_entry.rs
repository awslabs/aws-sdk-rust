// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_supplemental_tax_registration_entry(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::SupplementalTaxRegistrationEntry,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("registrationId").string(input.registration_id.as_str());
    }
    {
        object.key("registrationType").string(input.registration_type.as_str());
    }
    {
        object.key("legalName").string(input.legal_name.as_str());
    }
    if let Some(var_1) = &input.address {
        #[allow(unused_mut)]
        let mut object_2 = object.key("address").start_object();
        crate::protocol_serde::shape_address::ser_address(&mut object_2, var_1)?;
        object_2.finish();
    }
    Ok(())
}

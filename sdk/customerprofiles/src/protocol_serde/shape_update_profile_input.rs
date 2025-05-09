// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_profile_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_profile::UpdateProfileInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.account_number {
        object.key("AccountNumber").string(var_1.as_str());
    }
    if let Some(var_2) = &input.additional_information {
        object.key("AdditionalInformation").string(var_2.as_str());
    }
    if let Some(var_3) = &input.address {
        #[allow(unused_mut)]
        let mut object_4 = object.key("Address").start_object();
        crate::protocol_serde::shape_update_address::ser_update_address(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.attributes {
        #[allow(unused_mut)]
        let mut object_6 = object.key("Attributes").start_object();
        for (key_7, value_8) in var_5 {
            {
                object_6.key(key_7.as_str()).string(value_8.as_str());
            }
        }
        object_6.finish();
    }
    if let Some(var_9) = &input.billing_address {
        #[allow(unused_mut)]
        let mut object_10 = object.key("BillingAddress").start_object();
        crate::protocol_serde::shape_update_address::ser_update_address(&mut object_10, var_9)?;
        object_10.finish();
    }
    if let Some(var_11) = &input.birth_date {
        object.key("BirthDate").string(var_11.as_str());
    }
    if let Some(var_12) = &input.business_email_address {
        object.key("BusinessEmailAddress").string(var_12.as_str());
    }
    if let Some(var_13) = &input.business_name {
        object.key("BusinessName").string(var_13.as_str());
    }
    if let Some(var_14) = &input.business_phone_number {
        object.key("BusinessPhoneNumber").string(var_14.as_str());
    }
    if let Some(var_15) = &input.email_address {
        object.key("EmailAddress").string(var_15.as_str());
    }
    if let Some(var_16) = &input.first_name {
        object.key("FirstName").string(var_16.as_str());
    }
    if let Some(var_17) = &input.gender {
        object.key("Gender").string(var_17.as_str());
    }
    if let Some(var_18) = &input.gender_string {
        object.key("GenderString").string(var_18.as_str());
    }
    if let Some(var_19) = &input.home_phone_number {
        object.key("HomePhoneNumber").string(var_19.as_str());
    }
    if let Some(var_20) = &input.last_name {
        object.key("LastName").string(var_20.as_str());
    }
    if let Some(var_21) = &input.mailing_address {
        #[allow(unused_mut)]
        let mut object_22 = object.key("MailingAddress").start_object();
        crate::protocol_serde::shape_update_address::ser_update_address(&mut object_22, var_21)?;
        object_22.finish();
    }
    if let Some(var_23) = &input.middle_name {
        object.key("MiddleName").string(var_23.as_str());
    }
    if let Some(var_24) = &input.mobile_phone_number {
        object.key("MobilePhoneNumber").string(var_24.as_str());
    }
    if let Some(var_25) = &input.party_type {
        object.key("PartyType").string(var_25.as_str());
    }
    if let Some(var_26) = &input.party_type_string {
        object.key("PartyTypeString").string(var_26.as_str());
    }
    if let Some(var_27) = &input.personal_email_address {
        object.key("PersonalEmailAddress").string(var_27.as_str());
    }
    if let Some(var_28) = &input.phone_number {
        object.key("PhoneNumber").string(var_28.as_str());
    }
    if let Some(var_29) = &input.profile_id {
        object.key("ProfileId").string(var_29.as_str());
    }
    if let Some(var_30) = &input.shipping_address {
        #[allow(unused_mut)]
        let mut object_31 = object.key("ShippingAddress").start_object();
        crate::protocol_serde::shape_update_address::ser_update_address(&mut object_31, var_30)?;
        object_31.finish();
    }
    Ok(())
}

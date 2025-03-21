// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_transfer_domain_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::transfer_domain::TransferDomainInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.domain_name {
        object.key("DomainName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.idn_lang_code {
        object.key("IdnLangCode").string(var_2.as_str());
    }
    if let Some(var_3) = &input.duration_in_years {
        object.key("DurationInYears").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_3).into()),
        );
    }
    if let Some(var_4) = &input.nameservers {
        let mut array_5 = object.key("Nameservers").start_array();
        for item_6 in var_4 {
            {
                #[allow(unused_mut)]
                let mut object_7 = array_5.value().start_object();
                crate::protocol_serde::shape_nameserver::ser_nameserver(&mut object_7, item_6)?;
                object_7.finish();
            }
        }
        array_5.finish();
    }
    if let Some(var_8) = &input.auth_code {
        object.key("AuthCode").string(var_8.as_str());
    }
    if let Some(var_9) = &input.auto_renew {
        object.key("AutoRenew").boolean(*var_9);
    }
    if let Some(var_10) = &input.admin_contact {
        #[allow(unused_mut)]
        let mut object_11 = object.key("AdminContact").start_object();
        crate::protocol_serde::shape_contact_detail::ser_contact_detail(&mut object_11, var_10)?;
        object_11.finish();
    }
    if let Some(var_12) = &input.registrant_contact {
        #[allow(unused_mut)]
        let mut object_13 = object.key("RegistrantContact").start_object();
        crate::protocol_serde::shape_contact_detail::ser_contact_detail(&mut object_13, var_12)?;
        object_13.finish();
    }
    if let Some(var_14) = &input.tech_contact {
        #[allow(unused_mut)]
        let mut object_15 = object.key("TechContact").start_object();
        crate::protocol_serde::shape_contact_detail::ser_contact_detail(&mut object_15, var_14)?;
        object_15.finish();
    }
    if let Some(var_16) = &input.privacy_protect_admin_contact {
        object.key("PrivacyProtectAdminContact").boolean(*var_16);
    }
    if let Some(var_17) = &input.privacy_protect_registrant_contact {
        object.key("PrivacyProtectRegistrantContact").boolean(*var_17);
    }
    if let Some(var_18) = &input.privacy_protect_tech_contact {
        object.key("PrivacyProtectTechContact").boolean(*var_18);
    }
    if let Some(var_19) = &input.billing_contact {
        #[allow(unused_mut)]
        let mut object_20 = object.key("BillingContact").start_object();
        crate::protocol_serde::shape_contact_detail::ser_contact_detail(&mut object_20, var_19)?;
        object_20.finish();
    }
    if let Some(var_21) = &input.privacy_protect_billing_contact {
        object.key("PrivacyProtectBillingContact").boolean(*var_21);
    }
    Ok(())
}

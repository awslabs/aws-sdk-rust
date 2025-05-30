// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_user_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_user::UpdateUserInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.given_name {
        object.key("GivenName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.grant_poweruser_privileges {
        object.key("GrantPoweruserPrivileges").string(var_2.as_str());
    }
    if let Some(var_3) = &input.locale {
        object.key("Locale").string(var_3.as_str());
    }
    if let Some(var_4) = &input.storage_rule {
        #[allow(unused_mut)]
        let mut object_5 = object.key("StorageRule").start_object();
        crate::protocol_serde::shape_storage_rule_type::ser_storage_rule_type(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.surname {
        object.key("Surname").string(var_6.as_str());
    }
    if let Some(var_7) = &input.time_zone_id {
        object.key("TimeZoneId").string(var_7.as_str());
    }
    if let Some(var_8) = &input.r#type {
        object.key("Type").string(var_8.as_str());
    }
    Ok(())
}

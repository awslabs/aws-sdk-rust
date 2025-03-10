// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_identity_provider_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_identity_provider::UpdateIdentityProviderInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.user_pool_id {
        object.key("UserPoolId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.provider_name {
        object.key("ProviderName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.provider_details {
        #[allow(unused_mut)]
        let mut object_4 = object.key("ProviderDetails").start_object();
        for (key_5, value_6) in var_3 {
            {
                object_4.key(key_5.as_str()).string(value_6.as_str());
            }
        }
        object_4.finish();
    }
    if let Some(var_7) = &input.attribute_mapping {
        #[allow(unused_mut)]
        let mut object_8 = object.key("AttributeMapping").start_object();
        for (key_9, value_10) in var_7 {
            {
                object_8.key(key_9.as_str()).string(value_10.as_str());
            }
        }
        object_8.finish();
    }
    if let Some(var_11) = &input.idp_identifiers {
        let mut array_12 = object.key("IdpIdentifiers").start_array();
        for item_13 in var_11 {
            {
                array_12.value().string(item_13.as_str());
            }
        }
        array_12.finish();
    }
    Ok(())
}

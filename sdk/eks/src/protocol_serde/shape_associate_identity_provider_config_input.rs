// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_associate_identity_provider_config_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::associate_identity_provider_config::AssociateIdentityProviderConfigInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.client_request_token {
        object.key("clientRequestToken").string(var_1.as_str());
    }
    if let Some(var_2) = &input.oidc {
        #[allow(unused_mut)]
        let mut object_3 = object.key("oidc").start_object();
        crate::protocol_serde::shape_oidc_identity_provider_config_request::ser_oidc_identity_provider_config_request(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.tags {
        #[allow(unused_mut)]
        let mut object_5 = object.key("tags").start_object();
        for (key_6, value_7) in var_4 {
            {
                object_5.key(key_6.as_str()).string(value_7.as_str());
            }
        }
        object_5.finish();
    }
    Ok(())
}

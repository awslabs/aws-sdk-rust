// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_trusted_token_issuer_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_trusted_token_issuer::CreateTrustedTokenIssuerInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.instance_arn {
        object.key("InstanceArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.name {
        object.key("Name").string(var_2.as_str());
    }
    if let Some(var_3) = &input.trusted_token_issuer_type {
        object.key("TrustedTokenIssuerType").string(var_3.as_str());
    }
    if let Some(var_4) = &input.trusted_token_issuer_configuration {
        #[allow(unused_mut)]
        let mut object_5 = object.key("TrustedTokenIssuerConfiguration").start_object();
        crate::protocol_serde::shape_trusted_token_issuer_configuration::ser_trusted_token_issuer_configuration(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.client_token {
        object.key("ClientToken").string(var_6.as_str());
    }
    if let Some(var_7) = &input.tags {
        let mut array_8 = object.key("Tags").start_array();
        for item_9 in var_7 {
            {
                #[allow(unused_mut)]
                let mut object_10 = array_8.value().start_object();
                crate::protocol_serde::shape_tag::ser_tag(&mut object_10, item_9)?;
                object_10.finish();
            }
        }
        array_8.finish();
    }
    Ok(())
}

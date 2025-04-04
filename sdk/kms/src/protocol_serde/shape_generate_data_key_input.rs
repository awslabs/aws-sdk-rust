// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_generate_data_key_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::generate_data_key::GenerateDataKeyInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.key_id {
        object.key("KeyId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.encryption_context {
        #[allow(unused_mut)]
        let mut object_3 = object.key("EncryptionContext").start_object();
        for (key_4, value_5) in var_2 {
            {
                object_3.key(key_4.as_str()).string(value_5.as_str());
            }
        }
        object_3.finish();
    }
    if let Some(var_6) = &input.number_of_bytes {
        object.key("NumberOfBytes").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_6).into()),
        );
    }
    if let Some(var_7) = &input.key_spec {
        object.key("KeySpec").string(var_7.as_str());
    }
    if let Some(var_8) = &input.grant_tokens {
        let mut array_9 = object.key("GrantTokens").start_array();
        for item_10 in var_8 {
            {
                array_9.value().string(item_10.as_str());
            }
        }
        array_9.finish();
    }
    if let Some(var_11) = &input.recipient {
        #[allow(unused_mut)]
        let mut object_12 = object.key("Recipient").start_object();
        crate::protocol_serde::shape_recipient_info::ser_recipient_info(&mut object_12, var_11)?;
        object_12.finish();
    }
    if let Some(var_13) = &input.dry_run {
        object.key("DryRun").boolean(*var_13);
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_channel_subtype_config(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ChannelSubtypeConfig,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.telephony {
        #[allow(unused_mut)]
        let mut object_2 = object.key("telephony").start_object();
        crate::protocol_serde::shape_telephony_channel_subtype_config::ser_telephony_channel_subtype_config(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.sms {
        #[allow(unused_mut)]
        let mut object_4 = object.key("sms").start_object();
        crate::protocol_serde::shape_sms_channel_subtype_config::ser_sms_channel_subtype_config(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.email {
        #[allow(unused_mut)]
        let mut object_6 = object.key("email").start_object();
        crate::protocol_serde::shape_email_channel_subtype_config::ser_email_channel_subtype_config(&mut object_6, var_5)?;
        object_6.finish();
    }
    Ok(())
}

pub(crate) fn de_channel_subtype_config<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::ChannelSubtypeConfig>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::ChannelSubtypeConfigBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "telephony" => {
                            builder = builder.set_telephony(
                                crate::protocol_serde::shape_telephony_channel_subtype_config::de_telephony_channel_subtype_config(tokens)?,
                            );
                        }
                        "sms" => {
                            builder = builder.set_sms(crate::protocol_serde::shape_sms_channel_subtype_config::de_sms_channel_subtype_config(
                                tokens,
                            )?);
                        }
                        "email" => {
                            builder = builder
                                .set_email(crate::protocol_serde::shape_email_channel_subtype_config::de_email_channel_subtype_config(tokens)?);
                        }
                        _ => ::aws_smithy_json::deserialize::token::skip_value(tokens)?,
                    },
                    other => {
                        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(format!(
                            "expected object key or end object, found: {:?}",
                            other
                        )))
                    }
                }
            }
            Ok(Some(builder.build()))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

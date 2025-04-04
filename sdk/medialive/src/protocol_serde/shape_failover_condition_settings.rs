// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_failover_condition_settings(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::FailoverConditionSettings,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.audio_silence_settings {
        #[allow(unused_mut)]
        let mut object_2 = object.key("audioSilenceSettings").start_object();
        crate::protocol_serde::shape_audio_silence_failover_settings::ser_audio_silence_failover_settings(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.input_loss_settings {
        #[allow(unused_mut)]
        let mut object_4 = object.key("inputLossSettings").start_object();
        crate::protocol_serde::shape_input_loss_failover_settings::ser_input_loss_failover_settings(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.video_black_settings {
        #[allow(unused_mut)]
        let mut object_6 = object.key("videoBlackSettings").start_object();
        crate::protocol_serde::shape_video_black_failover_settings::ser_video_black_failover_settings(&mut object_6, var_5)?;
        object_6.finish();
    }
    Ok(())
}

pub(crate) fn de_failover_condition_settings<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::FailoverConditionSettings>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::FailoverConditionSettingsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "audioSilenceSettings" => {
                            builder = builder.set_audio_silence_settings(
                                crate::protocol_serde::shape_audio_silence_failover_settings::de_audio_silence_failover_settings(tokens)?,
                            );
                        }
                        "inputLossSettings" => {
                            builder = builder.set_input_loss_settings(
                                crate::protocol_serde::shape_input_loss_failover_settings::de_input_loss_failover_settings(tokens)?,
                            );
                        }
                        "videoBlackSettings" => {
                            builder = builder.set_video_black_settings(
                                crate::protocol_serde::shape_video_black_failover_settings::de_video_black_failover_settings(tokens)?,
                            );
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

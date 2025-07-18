// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_aac_settings(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::AacSettings,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.audio_description_broadcaster_mix {
        object.key("audioDescriptionBroadcasterMix").string(var_1.as_str());
    }
    if let Some(var_2) = &input.bitrate {
        object.key("bitrate").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_2).into()),
        );
    }
    if let Some(var_3) = &input.codec_profile {
        object.key("codecProfile").string(var_3.as_str());
    }
    if let Some(var_4) = &input.coding_mode {
        object.key("codingMode").string(var_4.as_str());
    }
    if let Some(var_5) = &input.loudness_measurement_mode {
        object.key("loudnessMeasurementMode").string(var_5.as_str());
    }
    if let Some(var_6) = &input.rap_interval {
        object.key("rapInterval").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_6).into()),
        );
    }
    if let Some(var_7) = &input.rate_control_mode {
        object.key("rateControlMode").string(var_7.as_str());
    }
    if let Some(var_8) = &input.raw_format {
        object.key("rawFormat").string(var_8.as_str());
    }
    if let Some(var_9) = &input.sample_rate {
        object.key("sampleRate").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_9).into()),
        );
    }
    if let Some(var_10) = &input.specification {
        object.key("specification").string(var_10.as_str());
    }
    if let Some(var_11) = &input.target_loudness_range {
        object.key("targetLoudnessRange").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_11).into()),
        );
    }
    if let Some(var_12) = &input.vbr_quality {
        object.key("vbrQuality").string(var_12.as_str());
    }
    Ok(())
}

pub(crate) fn de_aac_settings<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::AacSettings>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::AacSettingsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "audioDescriptionBroadcasterMix" => {
                            builder = builder.set_audio_description_broadcaster_mix(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| {
                                        s.to_unescaped()
                                            .map(|u| crate::types::AacAudioDescriptionBroadcasterMix::from(u.as_ref()))
                                    })
                                    .transpose()?,
                            );
                        }
                        "bitrate" => {
                            builder = builder.set_bitrate(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "codecProfile" => {
                            builder = builder.set_codec_profile(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::AacCodecProfile::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "codingMode" => {
                            builder = builder.set_coding_mode(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::AacCodingMode::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "loudnessMeasurementMode" => {
                            builder = builder.set_loudness_measurement_mode(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::AacLoudnessMeasurementMode::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "rapInterval" => {
                            builder = builder.set_rap_interval(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "rateControlMode" => {
                            builder = builder.set_rate_control_mode(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::AacRateControlMode::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "rawFormat" => {
                            builder = builder.set_raw_format(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::AacRawFormat::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "sampleRate" => {
                            builder = builder.set_sample_rate(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "specification" => {
                            builder = builder.set_specification(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::AacSpecification::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "targetLoudnessRange" => {
                            builder = builder.set_target_loudness_range(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "vbrQuality" => {
                            builder = builder.set_vbr_quality(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::AacVbrQuality::from(u.as_ref())))
                                    .transpose()?,
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

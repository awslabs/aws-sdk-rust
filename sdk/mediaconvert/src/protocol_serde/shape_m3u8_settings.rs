// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_m3u8_settings(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::M3u8Settings,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.audio_duration {
        object.key("audioDuration").string(var_1.as_str());
    }
    if let Some(var_2) = &input.audio_frames_per_pes {
        object.key("audioFramesPerPes").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_2).into()),
        );
    }
    if let Some(var_3) = &input.audio_pids {
        let mut array_4 = object.key("audioPids").start_array();
        for item_5 in var_3 {
            {
                array_4.value().number(
                    #[allow(clippy::useless_conversion)]
                    ::aws_smithy_types::Number::NegInt((*item_5).into()),
                );
            }
        }
        array_4.finish();
    }
    if let Some(var_6) = &input.audio_pts_offset_delta {
        object.key("audioPtsOffsetDelta").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_6).into()),
        );
    }
    if let Some(var_7) = &input.data_pts_control {
        object.key("dataPTSControl").string(var_7.as_str());
    }
    if let Some(var_8) = &input.max_pcr_interval {
        object.key("maxPcrInterval").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_8).into()),
        );
    }
    if let Some(var_9) = &input.nielsen_id3 {
        object.key("nielsenId3").string(var_9.as_str());
    }
    if let Some(var_10) = &input.pat_interval {
        object.key("patInterval").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_10).into()),
        );
    }
    if let Some(var_11) = &input.pcr_control {
        object.key("pcrControl").string(var_11.as_str());
    }
    if let Some(var_12) = &input.pcr_pid {
        object.key("pcrPid").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_12).into()),
        );
    }
    if let Some(var_13) = &input.pmt_interval {
        object.key("pmtInterval").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_13).into()),
        );
    }
    if let Some(var_14) = &input.pmt_pid {
        object.key("pmtPid").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_14).into()),
        );
    }
    if let Some(var_15) = &input.private_metadata_pid {
        object.key("privateMetadataPid").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_15).into()),
        );
    }
    if let Some(var_16) = &input.program_number {
        object.key("programNumber").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_16).into()),
        );
    }
    if let Some(var_17) = &input.pts_offset {
        object.key("ptsOffset").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_17).into()),
        );
    }
    if let Some(var_18) = &input.pts_offset_mode {
        object.key("ptsOffsetMode").string(var_18.as_str());
    }
    if let Some(var_19) = &input.scte35_pid {
        object.key("scte35Pid").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_19).into()),
        );
    }
    if let Some(var_20) = &input.scte35_source {
        object.key("scte35Source").string(var_20.as_str());
    }
    if let Some(var_21) = &input.timed_metadata {
        object.key("timedMetadata").string(var_21.as_str());
    }
    if let Some(var_22) = &input.timed_metadata_pid {
        object.key("timedMetadataPid").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_22).into()),
        );
    }
    if let Some(var_23) = &input.transport_stream_id {
        object.key("transportStreamId").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_23).into()),
        );
    }
    if let Some(var_24) = &input.video_pid {
        object.key("videoPid").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_24).into()),
        );
    }
    Ok(())
}

pub(crate) fn de_m3u8_settings<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::M3u8Settings>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::M3u8SettingsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "audioDuration" => {
                            builder = builder.set_audio_duration(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::M3u8AudioDuration::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "audioFramesPerPes" => {
                            builder = builder.set_audio_frames_per_pes(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "audioPids" => {
                            builder = builder.set_audio_pids(
                                crate::protocol_serde::shape_list_of_integer_min32_max8182::de_list_of_integer_min32_max8182(tokens)?,
                            );
                        }
                        "audioPtsOffsetDelta" => {
                            builder = builder.set_audio_pts_offset_delta(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "dataPTSControl" => {
                            builder = builder.set_data_pts_control(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::M3u8DataPtsControl::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "maxPcrInterval" => {
                            builder = builder.set_max_pcr_interval(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "nielsenId3" => {
                            builder = builder.set_nielsen_id3(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::M3u8NielsenId3::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "patInterval" => {
                            builder = builder.set_pat_interval(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "pcrControl" => {
                            builder = builder.set_pcr_control(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::M3u8PcrControl::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "pcrPid" => {
                            builder = builder.set_pcr_pid(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "pmtInterval" => {
                            builder = builder.set_pmt_interval(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "pmtPid" => {
                            builder = builder.set_pmt_pid(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "privateMetadataPid" => {
                            builder = builder.set_private_metadata_pid(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "programNumber" => {
                            builder = builder.set_program_number(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "ptsOffset" => {
                            builder = builder.set_pts_offset(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "ptsOffsetMode" => {
                            builder = builder.set_pts_offset_mode(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::TsPtsOffset::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "scte35Pid" => {
                            builder = builder.set_scte35_pid(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "scte35Source" => {
                            builder = builder.set_scte35_source(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::M3u8Scte35Source::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "timedMetadata" => {
                            builder = builder.set_timed_metadata(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::TimedMetadata::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "timedMetadataPid" => {
                            builder = builder.set_timed_metadata_pid(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "transportStreamId" => {
                            builder = builder.set_transport_stream_id(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "videoPid" => {
                            builder = builder.set_video_pid(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
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

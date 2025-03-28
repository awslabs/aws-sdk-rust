// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_call_analytics_job<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::CallAnalyticsJob>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::CallAnalyticsJobBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "CallAnalyticsJobName" => {
                            builder = builder.set_call_analytics_job_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "CallAnalyticsJobStatus" => {
                            builder = builder.set_call_analytics_job_status(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::CallAnalyticsJobStatus::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "CallAnalyticsJobDetails" => {
                            builder = builder.set_call_analytics_job_details(
                                crate::protocol_serde::shape_call_analytics_job_details::de_call_analytics_job_details(tokens)?,
                            );
                        }
                        "LanguageCode" => {
                            builder = builder.set_language_code(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::LanguageCode::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "MediaSampleRateHertz" => {
                            builder = builder.set_media_sample_rate_hertz(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "MediaFormat" => {
                            builder = builder.set_media_format(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::MediaFormat::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "Media" => {
                            builder = builder.set_media(crate::protocol_serde::shape_media::de_media(tokens)?);
                        }
                        "Transcript" => {
                            builder = builder.set_transcript(crate::protocol_serde::shape_transcript::de_transcript(tokens)?);
                        }
                        "StartTime" => {
                            builder = builder.set_start_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "CreationTime" => {
                            builder = builder.set_creation_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "CompletionTime" => {
                            builder = builder.set_completion_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "FailureReason" => {
                            builder = builder.set_failure_reason(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "DataAccessRoleArn" => {
                            builder = builder.set_data_access_role_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "IdentifiedLanguageScore" => {
                            builder = builder.set_identified_language_score(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?.map(|v| v.to_f32_lossy()),
                            );
                        }
                        "Settings" => {
                            builder = builder.set_settings(crate::protocol_serde::shape_call_analytics_job_settings::de_call_analytics_job_settings(
                                tokens,
                            )?);
                        }
                        "ChannelDefinitions" => {
                            builder =
                                builder.set_channel_definitions(crate::protocol_serde::shape_channel_definitions::de_channel_definitions(tokens)?);
                        }
                        "Tags" => {
                            builder = builder.set_tags(crate::protocol_serde::shape_tag_list::de_tag_list(tokens)?);
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

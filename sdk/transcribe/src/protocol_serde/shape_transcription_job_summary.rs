// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_transcription_job_summary<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::TranscriptionJobSummary>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::TranscriptionJobSummaryBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "TranscriptionJobName" => {
                            builder = builder.set_transcription_job_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "CreationTime" => {
                            builder = builder.set_creation_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "StartTime" => {
                            builder = builder.set_start_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
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
                        "LanguageCode" => {
                            builder = builder.set_language_code(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::LanguageCode::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "TranscriptionJobStatus" => {
                            builder = builder.set_transcription_job_status(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::TranscriptionJobStatus::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "FailureReason" => {
                            builder = builder.set_failure_reason(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "OutputLocationType" => {
                            builder = builder.set_output_location_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::OutputLocationType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "ContentRedaction" => {
                            builder = builder.set_content_redaction(crate::protocol_serde::shape_content_redaction::de_content_redaction(tokens)?);
                        }
                        "ModelSettings" => {
                            builder = builder.set_model_settings(crate::protocol_serde::shape_model_settings::de_model_settings(tokens)?);
                        }
                        "IdentifyLanguage" => {
                            builder = builder.set_identify_language(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "IdentifyMultipleLanguages" => {
                            builder =
                                builder.set_identify_multiple_languages(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "IdentifiedLanguageScore" => {
                            builder = builder.set_identified_language_score(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?.map(|v| v.to_f32_lossy()),
                            );
                        }
                        "LanguageCodes" => {
                            builder = builder.set_language_codes(crate::protocol_serde::shape_language_code_list::de_language_code_list(tokens)?);
                        }
                        "ToxicityDetection" => {
                            builder = builder.set_toxicity_detection(crate::protocol_serde::shape_toxicity_detection::de_toxicity_detection(tokens)?);
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

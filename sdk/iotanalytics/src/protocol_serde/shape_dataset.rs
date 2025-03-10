// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_dataset<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::Dataset>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::DatasetBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "name" => {
                            builder = builder.set_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "arn" => {
                            builder = builder.set_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "actions" => {
                            builder = builder.set_actions(crate::protocol_serde::shape_dataset_actions::de_dataset_actions(tokens)?);
                        }
                        "triggers" => {
                            builder = builder.set_triggers(crate::protocol_serde::shape_dataset_triggers::de_dataset_triggers(tokens)?);
                        }
                        "contentDeliveryRules" => {
                            builder = builder.set_content_delivery_rules(
                                crate::protocol_serde::shape_dataset_content_delivery_rules::de_dataset_content_delivery_rules(tokens)?,
                            );
                        }
                        "status" => {
                            builder = builder.set_status(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::DatasetStatus::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "creationTime" => {
                            builder = builder.set_creation_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "lastUpdateTime" => {
                            builder = builder.set_last_update_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "retentionPeriod" => {
                            builder = builder.set_retention_period(crate::protocol_serde::shape_retention_period::de_retention_period(tokens)?);
                        }
                        "versioningConfiguration" => {
                            builder = builder.set_versioning_configuration(
                                crate::protocol_serde::shape_versioning_configuration::de_versioning_configuration(tokens)?,
                            );
                        }
                        "lateDataRules" => {
                            builder = builder.set_late_data_rules(crate::protocol_serde::shape_late_data_rules::de_late_data_rules(tokens)?);
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

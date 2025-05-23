// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_data_migration<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::DataMigration>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::DataMigrationBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "DataMigrationName" => {
                            builder = builder.set_data_migration_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "DataMigrationArn" => {
                            builder = builder.set_data_migration_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "DataMigrationCreateTime" => {
                            builder = builder.set_data_migration_create_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::DateTimeWithOffset,
                            )?);
                        }
                        "DataMigrationStartTime" => {
                            builder = builder.set_data_migration_start_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::DateTimeWithOffset,
                            )?);
                        }
                        "DataMigrationEndTime" => {
                            builder = builder.set_data_migration_end_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::DateTimeWithOffset,
                            )?);
                        }
                        "ServiceAccessRoleArn" => {
                            builder = builder.set_service_access_role_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "MigrationProjectArn" => {
                            builder = builder.set_migration_project_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "DataMigrationType" => {
                            builder = builder.set_data_migration_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::MigrationTypeValue::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "DataMigrationSettings" => {
                            builder = builder.set_data_migration_settings(
                                crate::protocol_serde::shape_data_migration_settings::de_data_migration_settings(tokens)?,
                            );
                        }
                        "SourceDataSettings" => {
                            builder =
                                builder.set_source_data_settings(crate::protocol_serde::shape_source_data_settings::de_source_data_settings(tokens)?);
                        }
                        "TargetDataSettings" => {
                            builder =
                                builder.set_target_data_settings(crate::protocol_serde::shape_target_data_settings::de_target_data_settings(tokens)?);
                        }
                        "DataMigrationStatistics" => {
                            builder = builder.set_data_migration_statistics(
                                crate::protocol_serde::shape_data_migration_statistics::de_data_migration_statistics(tokens)?,
                            );
                        }
                        "DataMigrationStatus" => {
                            builder = builder.set_data_migration_status(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "PublicIpAddresses" => {
                            builder = builder
                                .set_public_ip_addresses(crate::protocol_serde::shape_public_ip_address_list::de_public_ip_address_list(tokens)?);
                        }
                        "DataMigrationCidrBlocks" => {
                            builder = builder.set_data_migration_cidr_blocks(
                                crate::protocol_serde::shape_data_migration_cidr_block::de_data_migration_cidr_block(tokens)?,
                            );
                        }
                        "LastFailureMessage" => {
                            builder = builder.set_last_failure_message(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "StopReason" => {
                            builder = builder.set_stop_reason(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
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

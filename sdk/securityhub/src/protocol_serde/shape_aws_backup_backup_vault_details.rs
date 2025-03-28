// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_aws_backup_backup_vault_details(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::AwsBackupBackupVaultDetails,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.backup_vault_arn {
        object.key("BackupVaultArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.backup_vault_name {
        object.key("BackupVaultName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.encryption_key_arn {
        object.key("EncryptionKeyArn").string(var_3.as_str());
    }
    if let Some(var_4) = &input.notifications {
        #[allow(unused_mut)]
        let mut object_5 = object.key("Notifications").start_object();
        crate::protocol_serde::shape_aws_backup_backup_vault_notifications_details::ser_aws_backup_backup_vault_notifications_details(
            &mut object_5,
            var_4,
        )?;
        object_5.finish();
    }
    if let Some(var_6) = &input.access_policy {
        object.key("AccessPolicy").string(var_6.as_str());
    }
    Ok(())
}

pub(crate) fn de_aws_backup_backup_vault_details<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::AwsBackupBackupVaultDetails>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::AwsBackupBackupVaultDetailsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "BackupVaultArn" => {
                            builder = builder.set_backup_vault_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "BackupVaultName" => {
                            builder = builder.set_backup_vault_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "EncryptionKeyArn" => {
                            builder = builder.set_encryption_key_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "Notifications" => {
                            builder = builder.set_notifications(
                                    crate::protocol_serde::shape_aws_backup_backup_vault_notifications_details::de_aws_backup_backup_vault_notifications_details(tokens)?
                                );
                        }
                        "AccessPolicy" => {
                            builder = builder.set_access_policy(
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

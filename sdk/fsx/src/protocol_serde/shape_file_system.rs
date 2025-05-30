// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_file_system<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::FileSystem>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::FileSystemBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "OwnerId" => {
                            builder = builder.set_owner_id(
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
                        "FileSystemId" => {
                            builder = builder.set_file_system_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "FileSystemType" => {
                            builder = builder.set_file_system_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::FileSystemType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "Lifecycle" => {
                            builder = builder.set_lifecycle(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::FileSystemLifecycle::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "FailureDetails" => {
                            builder = builder.set_failure_details(
                                crate::protocol_serde::shape_file_system_failure_details::de_file_system_failure_details(tokens)?,
                            );
                        }
                        "StorageCapacity" => {
                            builder = builder.set_storage_capacity(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "StorageType" => {
                            builder = builder.set_storage_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::StorageType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "VpcId" => {
                            builder = builder.set_vpc_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "SubnetIds" => {
                            builder = builder.set_subnet_ids(crate::protocol_serde::shape_subnet_ids::de_subnet_ids(tokens)?);
                        }
                        "NetworkInterfaceIds" => {
                            builder = builder
                                .set_network_interface_ids(crate::protocol_serde::shape_network_interface_ids::de_network_interface_ids(tokens)?);
                        }
                        "DNSName" => {
                            builder = builder.set_dns_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "KmsKeyId" => {
                            builder = builder.set_kms_key_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "ResourceARN" => {
                            builder = builder.set_resource_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "Tags" => {
                            builder = builder.set_tags(crate::protocol_serde::shape_tags::de_tags(tokens)?);
                        }
                        "WindowsConfiguration" => {
                            builder = builder.set_windows_configuration(
                                crate::protocol_serde::shape_windows_file_system_configuration::de_windows_file_system_configuration(tokens)?,
                            );
                        }
                        "LustreConfiguration" => {
                            builder = builder.set_lustre_configuration(
                                crate::protocol_serde::shape_lustre_file_system_configuration::de_lustre_file_system_configuration(tokens)?,
                            );
                        }
                        "AdministrativeActions" => {
                            builder = builder
                                .set_administrative_actions(crate::protocol_serde::shape_administrative_actions::de_administrative_actions(tokens)?);
                        }
                        "OntapConfiguration" => {
                            builder = builder.set_ontap_configuration(
                                crate::protocol_serde::shape_ontap_file_system_configuration::de_ontap_file_system_configuration(tokens)?,
                            );
                        }
                        "FileSystemTypeVersion" => {
                            builder = builder.set_file_system_type_version(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "OpenZFSConfiguration" => {
                            builder = builder.set_open_zfs_configuration(
                                crate::protocol_serde::shape_open_zfs_file_system_configuration::de_open_zfs_file_system_configuration(tokens)?,
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

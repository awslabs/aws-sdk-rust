// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_data_repository_association<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::DataRepositoryAssociation>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::DataRepositoryAssociationBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "AssociationId" => {
                            builder = builder.set_association_id(
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
                        "FileSystemId" => {
                            builder = builder.set_file_system_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "Lifecycle" => {
                            builder = builder.set_lifecycle(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::DataRepositoryLifecycle::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "FailureDetails" => {
                            builder = builder.set_failure_details(
                                crate::protocol_serde::shape_data_repository_failure_details::de_data_repository_failure_details(tokens)?,
                            );
                        }
                        "FileSystemPath" => {
                            builder = builder.set_file_system_path(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "DataRepositoryPath" => {
                            builder = builder.set_data_repository_path(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "BatchImportMetaDataOnCreate" => {
                            builder = builder
                                .set_batch_import_meta_data_on_create(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "ImportedFileChunkSize" => {
                            builder = builder.set_imported_file_chunk_size(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "S3" => {
                            builder = builder
                                .set_s3(crate::protocol_serde::shape_s3_data_repository_configuration::de_s3_data_repository_configuration(tokens)?);
                        }
                        "Tags" => {
                            builder = builder.set_tags(crate::protocol_serde::shape_tags::de_tags(tokens)?);
                        }
                        "CreationTime" => {
                            builder = builder.set_creation_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "FileCacheId" => {
                            builder = builder.set_file_cache_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "FileCachePath" => {
                            builder = builder.set_file_cache_path(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "DataRepositorySubdirectories" => {
                            builder = builder.set_data_repository_subdirectories(
                                crate::protocol_serde::shape_sub_directories_paths::de_sub_directories_paths(tokens)?,
                            );
                        }
                        "NFS" => {
                            builder = builder.set_nfs(
                                crate::protocol_serde::shape_nfs_data_repository_configuration::de_nfs_data_repository_configuration(tokens)?,
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

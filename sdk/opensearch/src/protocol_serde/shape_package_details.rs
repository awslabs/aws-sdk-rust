// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_package_details<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::PackageDetails>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::PackageDetailsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "PackageID" => {
                            builder = builder.set_package_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "PackageName" => {
                            builder = builder.set_package_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "PackageType" => {
                            builder = builder.set_package_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::PackageType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "PackageDescription" => {
                            builder = builder.set_package_description(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "PackageStatus" => {
                            builder = builder.set_package_status(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::PackageStatus::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "CreatedAt" => {
                            builder = builder.set_created_at(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "LastUpdatedAt" => {
                            builder = builder.set_last_updated_at(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "AvailablePackageVersion" => {
                            builder = builder.set_available_package_version(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "ErrorDetails" => {
                            builder = builder.set_error_details(crate::protocol_serde::shape_error_details::de_error_details(tokens)?);
                        }
                        "EngineVersion" => {
                            builder = builder.set_engine_version(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "AvailablePluginProperties" => {
                            builder = builder
                                .set_available_plugin_properties(crate::protocol_serde::shape_plugin_properties::de_plugin_properties(tokens)?);
                        }
                        "AvailablePackageConfiguration" => {
                            builder = builder.set_available_package_configuration(
                                crate::protocol_serde::shape_package_configuration::de_package_configuration(tokens)?,
                            );
                        }
                        "AllowListedUserList" => {
                            builder =
                                builder.set_allow_listed_user_list(crate::protocol_serde::shape_package_user_list::de_package_user_list(tokens)?);
                        }
                        "PackageOwner" => {
                            builder = builder.set_package_owner(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "PackageVendingOptions" => {
                            builder = builder.set_package_vending_options(
                                crate::protocol_serde::shape_package_vending_options::de_package_vending_options(tokens)?,
                            );
                        }
                        "PackageEncryptionOptions" => {
                            builder = builder.set_package_encryption_options(
                                crate::protocol_serde::shape_package_encryption_options::de_package_encryption_options(tokens)?,
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

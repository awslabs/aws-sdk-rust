// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_provisioned<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::Provisioned>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::ProvisionedBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "brokerNodeGroupInfo" => {
                            builder = builder
                                .set_broker_node_group_info(crate::protocol_serde::shape_broker_node_group_info::de_broker_node_group_info(tokens)?);
                        }
                        "currentBrokerSoftwareInfo" => {
                            builder = builder.set_current_broker_software_info(
                                crate::protocol_serde::shape_broker_software_info::de_broker_software_info(tokens)?,
                            );
                        }
                        "clientAuthentication" => {
                            builder = builder
                                .set_client_authentication(crate::protocol_serde::shape_client_authentication::de_client_authentication(tokens)?);
                        }
                        "encryptionInfo" => {
                            builder = builder.set_encryption_info(crate::protocol_serde::shape_encryption_info::de_encryption_info(tokens)?);
                        }
                        "enhancedMonitoring" => {
                            builder = builder.set_enhanced_monitoring(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::EnhancedMonitoring::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "openMonitoring" => {
                            builder =
                                builder.set_open_monitoring(crate::protocol_serde::shape_open_monitoring_info::de_open_monitoring_info(tokens)?);
                        }
                        "loggingInfo" => {
                            builder = builder.set_logging_info(crate::protocol_serde::shape_logging_info::de_logging_info(tokens)?);
                        }
                        "numberOfBrokerNodes" => {
                            builder = builder.set_number_of_broker_nodes(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "zookeeperConnectString" => {
                            builder = builder.set_zookeeper_connect_string(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "zookeeperConnectStringTls" => {
                            builder = builder.set_zookeeper_connect_string_tls(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "storageMode" => {
                            builder = builder.set_storage_mode(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::StorageMode::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "customerActionStatus" => {
                            builder = builder.set_customer_action_status(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::CustomerActionStatus::from(u.as_ref())))
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
            Ok(Some(crate::serde_util::provisioned_correct_errors(builder).build()))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

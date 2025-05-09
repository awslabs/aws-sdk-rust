// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_input<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::Input>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::InputBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "arn" => {
                            builder = builder.set_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "attachedChannels" => {
                            builder = builder.set_attached_channels(crate::protocol_serde::shape_list_of_string::de_list_of_string(tokens)?);
                        }
                        "destinations" => {
                            builder = builder.set_destinations(crate::protocol_serde::shape_list_of_input_destination::de_list_of_input_destination(
                                tokens,
                            )?);
                        }
                        "id" => {
                            builder = builder.set_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "inputClass" => {
                            builder = builder.set_input_class(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::InputClass::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "inputDevices" => {
                            builder = builder.set_input_devices(
                                crate::protocol_serde::shape_list_of_input_device_settings::de_list_of_input_device_settings(tokens)?,
                            );
                        }
                        "inputPartnerIds" => {
                            builder = builder.set_input_partner_ids(crate::protocol_serde::shape_list_of_string::de_list_of_string(tokens)?);
                        }
                        "inputSourceType" => {
                            builder = builder.set_input_source_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::InputSourceType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "mediaConnectFlows" => {
                            builder = builder.set_media_connect_flows(
                                crate::protocol_serde::shape_list_of_media_connect_flow::de_list_of_media_connect_flow(tokens)?,
                            );
                        }
                        "name" => {
                            builder = builder.set_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "roleArn" => {
                            builder = builder.set_role_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "securityGroups" => {
                            builder = builder.set_security_groups(crate::protocol_serde::shape_list_of_string::de_list_of_string(tokens)?);
                        }
                        "sources" => {
                            builder = builder.set_sources(crate::protocol_serde::shape_list_of_input_source::de_list_of_input_source(tokens)?);
                        }
                        "state" => {
                            builder = builder.set_state(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::InputState::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "tags" => {
                            builder = builder.set_tags(crate::protocol_serde::shape_tags::de_tags(tokens)?);
                        }
                        "type" => {
                            builder = builder.set_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::InputType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "srtSettings" => {
                            builder = builder.set_srt_settings(crate::protocol_serde::shape_srt_settings::de_srt_settings(tokens)?);
                        }
                        "inputNetworkLocation" => {
                            builder = builder.set_input_network_location(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::InputNetworkLocation::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "multicastSettings" => {
                            builder = builder.set_multicast_settings(crate::protocol_serde::shape_multicast_settings::de_multicast_settings(tokens)?);
                        }
                        "smpte2110ReceiverGroupSettings" => {
                            builder = builder.set_smpte2110_receiver_group_settings(
                                crate::protocol_serde::shape_smpte2110_receiver_group_settings::de_smpte2110_receiver_group_settings(tokens)?,
                            );
                        }
                        "sdiSources" => {
                            builder = builder.set_sdi_sources(crate::protocol_serde::shape_input_sdi_sources::de_input_sdi_sources(tokens)?);
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

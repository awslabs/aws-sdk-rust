// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_origin_endpoint<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::OriginEndpoint>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::OriginEndpointBuilder::default();
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
                        "authorization" => {
                            builder = builder.set_authorization(crate::protocol_serde::shape_authorization::de_authorization(tokens)?);
                        }
                        "channelId" => {
                            builder = builder.set_channel_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "cmafPackage" => {
                            builder = builder.set_cmaf_package(crate::protocol_serde::shape_cmaf_package::de_cmaf_package(tokens)?);
                        }
                        "createdAt" => {
                            builder = builder.set_created_at(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "dashPackage" => {
                            builder = builder.set_dash_package(crate::protocol_serde::shape_dash_package::de_dash_package(tokens)?);
                        }
                        "description" => {
                            builder = builder.set_description(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "hlsPackage" => {
                            builder = builder.set_hls_package(crate::protocol_serde::shape_hls_package::de_hls_package(tokens)?);
                        }
                        "id" => {
                            builder = builder.set_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "manifestName" => {
                            builder = builder.set_manifest_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "mssPackage" => {
                            builder = builder.set_mss_package(crate::protocol_serde::shape_mss_package::de_mss_package(tokens)?);
                        }
                        "origination" => {
                            builder = builder.set_origination(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::Origination::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "startoverWindowSeconds" => {
                            builder = builder.set_startover_window_seconds(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "tags" => {
                            builder = builder.set_tags(crate::protocol_serde::shape_tags::de_tags(tokens)?);
                        }
                        "timeDelaySeconds" => {
                            builder = builder.set_time_delay_seconds(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "url" => {
                            builder = builder.set_url(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "whitelist" => {
                            builder = builder.set_whitelist(crate::protocol_serde::shape_list_of_string::de_list_of_string(tokens)?);
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

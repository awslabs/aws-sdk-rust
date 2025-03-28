// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_sapo_data_connector_profile_properties(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::SapoDataConnectorProfileProperties,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("applicationHostUrl").string(input.application_host_url.as_str());
    }
    {
        object.key("applicationServicePath").string(input.application_service_path.as_str());
    }
    {
        object.key("portNumber").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.port_number).into()),
        );
    }
    {
        object.key("clientNumber").string(input.client_number.as_str());
    }
    if let Some(var_1) = &input.logon_language {
        object.key("logonLanguage").string(var_1.as_str());
    }
    if let Some(var_2) = &input.private_link_service_name {
        object.key("privateLinkServiceName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.o_auth_properties {
        #[allow(unused_mut)]
        let mut object_4 = object.key("oAuthProperties").start_object();
        crate::protocol_serde::shape_o_auth_properties::ser_o_auth_properties(&mut object_4, var_3)?;
        object_4.finish();
    }
    if input.disable_sso {
        object.key("disableSSO").boolean(input.disable_sso);
    }
    Ok(())
}

pub(crate) fn de_sapo_data_connector_profile_properties<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::SapoDataConnectorProfileProperties>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::SapoDataConnectorProfilePropertiesBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "applicationHostUrl" => {
                            builder = builder.set_application_host_url(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "applicationServicePath" => {
                            builder = builder.set_application_service_path(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "portNumber" => {
                            builder = builder.set_port_number(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "clientNumber" => {
                            builder = builder.set_client_number(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "logonLanguage" => {
                            builder = builder.set_logon_language(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "privateLinkServiceName" => {
                            builder = builder.set_private_link_service_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "oAuthProperties" => {
                            builder = builder.set_o_auth_properties(crate::protocol_serde::shape_o_auth_properties::de_o_auth_properties(tokens)?);
                        }
                        "disableSSO" => {
                            builder = builder.set_disable_sso(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
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
            Ok(Some(
                crate::serde_util::sapo_data_connector_profile_properties_correct_errors(builder)
                    .build()
                    .map_err(|err| ::aws_smithy_json::deserialize::error::DeserializeError::custom_source("Response was invalid", err))?,
            ))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

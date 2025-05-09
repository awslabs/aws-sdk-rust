// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_cors_rule(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::CorsRule,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        let mut array_1 = object.key("AllowedOrigins").start_array();
        for item_2 in &input.allowed_origins {
            {
                array_1.value().string(item_2.as_str());
            }
        }
        array_1.finish();
    }
    if let Some(var_3) = &input.allowed_methods {
        let mut array_4 = object.key("AllowedMethods").start_array();
        for item_5 in var_3 {
            {
                array_4.value().string(item_5.as_str());
            }
        }
        array_4.finish();
    }
    {
        let mut array_6 = object.key("AllowedHeaders").start_array();
        for item_7 in &input.allowed_headers {
            {
                array_6.value().string(item_7.as_str());
            }
        }
        array_6.finish();
    }
    if input.max_age_seconds != 0 {
        object.key("MaxAgeSeconds").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.max_age_seconds).into()),
        );
    }
    if let Some(var_8) = &input.expose_headers {
        let mut array_9 = object.key("ExposeHeaders").start_array();
        for item_10 in var_8 {
            {
                array_9.value().string(item_10.as_str());
            }
        }
        array_9.finish();
    }
    Ok(())
}

pub(crate) fn de_cors_rule<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::CorsRule>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::CorsRuleBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "AllowedOrigins" => {
                            builder = builder.set_allowed_origins(crate::protocol_serde::shape_allowed_origins::de_allowed_origins(tokens)?);
                        }
                        "AllowedMethods" => {
                            builder = builder.set_allowed_methods(crate::protocol_serde::shape_allowed_methods::de_allowed_methods(tokens)?);
                        }
                        "AllowedHeaders" => {
                            builder = builder.set_allowed_headers(crate::protocol_serde::shape_allowed_headers::de_allowed_headers(tokens)?);
                        }
                        "MaxAgeSeconds" => {
                            builder = builder.set_max_age_seconds(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "ExposeHeaders" => {
                            builder = builder.set_expose_headers(crate::protocol_serde::shape_expose_headers::de_expose_headers(tokens)?);
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
            Ok(Some(crate::serde_util::cors_rule_correct_errors(builder).build().map_err(|err| {
                ::aws_smithy_json::deserialize::error::DeserializeError::custom_source("Response was invalid", err)
            })?))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

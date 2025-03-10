// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_in_app_message_content(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::InAppMessageContent,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.background_color {
        object.key("BackgroundColor").string(var_1.as_str());
    }
    if let Some(var_2) = &input.body_config {
        #[allow(unused_mut)]
        let mut object_3 = object.key("BodyConfig").start_object();
        crate::protocol_serde::shape_in_app_message_body_config::ser_in_app_message_body_config(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.header_config {
        #[allow(unused_mut)]
        let mut object_5 = object.key("HeaderConfig").start_object();
        crate::protocol_serde::shape_in_app_message_header_config::ser_in_app_message_header_config(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.image_url {
        object.key("ImageUrl").string(var_6.as_str());
    }
    if let Some(var_7) = &input.primary_btn {
        #[allow(unused_mut)]
        let mut object_8 = object.key("PrimaryBtn").start_object();
        crate::protocol_serde::shape_in_app_message_button::ser_in_app_message_button(&mut object_8, var_7)?;
        object_8.finish();
    }
    if let Some(var_9) = &input.secondary_btn {
        #[allow(unused_mut)]
        let mut object_10 = object.key("SecondaryBtn").start_object();
        crate::protocol_serde::shape_in_app_message_button::ser_in_app_message_button(&mut object_10, var_9)?;
        object_10.finish();
    }
    Ok(())
}

pub(crate) fn de_in_app_message_content<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::InAppMessageContent>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::InAppMessageContentBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "BackgroundColor" => {
                            builder = builder.set_background_color(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "BodyConfig" => {
                            builder = builder.set_body_config(
                                crate::protocol_serde::shape_in_app_message_body_config::de_in_app_message_body_config(tokens)?,
                            );
                        }
                        "HeaderConfig" => {
                            builder = builder.set_header_config(
                                crate::protocol_serde::shape_in_app_message_header_config::de_in_app_message_header_config(tokens)?,
                            );
                        }
                        "ImageUrl" => {
                            builder = builder.set_image_url(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "PrimaryBtn" => {
                            builder = builder.set_primary_btn(crate::protocol_serde::shape_in_app_message_button::de_in_app_message_button(tokens)?);
                        }
                        "SecondaryBtn" => {
                            builder =
                                builder.set_secondary_btn(crate::protocol_serde::shape_in_app_message_button::de_in_app_message_button(tokens)?);
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

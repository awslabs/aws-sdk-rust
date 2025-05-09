// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_medical_result<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::MedicalResult>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::MedicalResultBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "ResultId" => {
                            builder = builder.set_result_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "StartTime" => {
                            builder = builder.set_start_time(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?.map(|v| v.to_f64_lossy()),
                            );
                        }
                        "EndTime" => {
                            builder = builder
                                .set_end_time(::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?.map(|v| v.to_f64_lossy()));
                        }
                        "IsPartial" => {
                            builder = builder.set_is_partial(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "Alternatives" => {
                            builder = builder.set_alternatives(crate::protocol_serde::shape_medical_alternative_list::de_medical_alternative_list(
                                tokens,
                            )?);
                        }
                        "ChannelId" => {
                            builder = builder.set_channel_id(
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

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_parameter_value(
    object_3: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ParameterValue,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    match input {
        crate::types::ParameterValue::Integer(inner) => {
            object_3.key("Integer").number(
                #[allow(clippy::useless_conversion)]
                ::aws_smithy_types::Number::NegInt((*inner).into()),
            );
        }
        crate::types::ParameterValue::IntegerList(inner) => {
            let mut array_1 = object_3.key("IntegerList").start_array();
            for item_2 in inner {
                {
                    array_1.value().number(
                        #[allow(clippy::useless_conversion)]
                        ::aws_smithy_types::Number::NegInt((*item_2).into()),
                    );
                }
            }
            array_1.finish();
        }
        crate::types::ParameterValue::Double(inner) => {
            object_3.key("Double").number(
                #[allow(clippy::useless_conversion)]
                ::aws_smithy_types::Number::Float((*inner).into()),
            );
        }
        crate::types::ParameterValue::String(inner) => {
            object_3.key("String").string(inner.as_str());
        }
        crate::types::ParameterValue::StringList(inner) => {
            let mut array_3 = object_3.key("StringList").start_array();
            for item_4 in inner {
                {
                    array_3.value().string(item_4.as_str());
                }
            }
            array_3.finish();
        }
        crate::types::ParameterValue::Boolean(inner) => {
            object_3.key("Boolean").boolean(*inner);
        }
        crate::types::ParameterValue::Enum(inner) => {
            object_3.key("Enum").string(inner.as_str());
        }
        crate::types::ParameterValue::EnumList(inner) => {
            let mut array_5 = object_3.key("EnumList").start_array();
            for item_6 in inner {
                {
                    array_5.value().string(item_6.as_str());
                }
            }
            array_5.finish();
        }
        crate::types::ParameterValue::Unknown => {
            return Err(::aws_smithy_types::error::operation::SerializationError::unknown_variant(
                "ParameterValue",
            ))
        }
    }
    Ok(())
}

pub(crate) fn de_parameter_value<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::ParameterValue>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    let mut variant = None;
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => return Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => loop {
            match tokens.next().transpose()? {
                Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => {
                    if let ::std::option::Option::Some(::std::result::Result::Ok(::aws_smithy_json::deserialize::Token::ValueNull { .. })) =
                        tokens.peek()
                    {
                        let _ = tokens.next().expect("peek returned a token")?;
                        continue;
                    }
                    let key = key.to_unescaped()?;
                    if key == "__type" {
                        ::aws_smithy_json::deserialize::token::skip_value(tokens)?;
                        continue;
                    }
                    if variant.is_some() {
                        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
                            "encountered mixed variants in union",
                        ));
                    }
                    variant = match key.as_ref() {
                        "Integer" => Some(crate::types::ParameterValue::Integer(
                            ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                .map(i32::try_from)
                                .transpose()?
                                .ok_or_else(|| {
                                    ::aws_smithy_json::deserialize::error::DeserializeError::custom("value for 'Integer' cannot be null")
                                })?,
                        )),
                        "IntegerList" => Some(crate::types::ParameterValue::IntegerList(
                            crate::protocol_serde::shape_integer_list::de_integer_list(tokens)?.ok_or_else(|| {
                                ::aws_smithy_json::deserialize::error::DeserializeError::custom("value for 'IntegerList' cannot be null")
                            })?,
                        )),
                        "Double" => Some(crate::types::ParameterValue::Double(
                            ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                .map(|v| v.to_f64_lossy())
                                .ok_or_else(|| {
                                    ::aws_smithy_json::deserialize::error::DeserializeError::custom("value for 'Double' cannot be null")
                                })?,
                        )),
                        "String" => Some(crate::types::ParameterValue::String(
                            ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                .transpose()?
                                .ok_or_else(|| {
                                    ::aws_smithy_json::deserialize::error::DeserializeError::custom("value for 'String' cannot be null")
                                })?,
                        )),
                        "StringList" => Some(crate::types::ParameterValue::StringList(
                            crate::protocol_serde::shape_string_list::de_string_list(tokens)?.ok_or_else(|| {
                                ::aws_smithy_json::deserialize::error::DeserializeError::custom("value for 'StringList' cannot be null")
                            })?,
                        )),
                        "Boolean" => Some(crate::types::ParameterValue::Boolean(
                            ::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?.ok_or_else(|| {
                                ::aws_smithy_json::deserialize::error::DeserializeError::custom("value for 'Boolean' cannot be null")
                            })?,
                        )),
                        "Enum" => Some(crate::types::ParameterValue::Enum(
                            ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                .transpose()?
                                .ok_or_else(|| ::aws_smithy_json::deserialize::error::DeserializeError::custom("value for 'Enum' cannot be null"))?,
                        )),
                        "EnumList" => Some(crate::types::ParameterValue::EnumList(
                            crate::protocol_serde::shape_string_list::de_string_list(tokens)?.ok_or_else(|| {
                                ::aws_smithy_json::deserialize::error::DeserializeError::custom("value for 'EnumList' cannot be null")
                            })?,
                        )),
                        _ => {
                            ::aws_smithy_json::deserialize::token::skip_value(tokens)?;
                            Some(crate::types::ParameterValue::Unknown)
                        }
                    };
                }
                other => {
                    return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(format!(
                        "expected object key or end object, found: {:?}",
                        other
                    )))
                }
            }
        },
        _ => {
            return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
                "expected start object or null",
            ))
        }
    }
    if variant.is_none() {
        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "Union did not contain a valid variant.",
        ));
    }
    Ok(variant)
}

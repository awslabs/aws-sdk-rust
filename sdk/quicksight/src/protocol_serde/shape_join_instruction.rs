// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_join_instruction(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::JoinInstruction,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("LeftOperand").string(input.left_operand.as_str());
    }
    {
        object.key("RightOperand").string(input.right_operand.as_str());
    }
    if let Some(var_1) = &input.left_join_key_properties {
        #[allow(unused_mut)]
        let mut object_2 = object.key("LeftJoinKeyProperties").start_object();
        crate::protocol_serde::shape_join_key_properties::ser_join_key_properties(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.right_join_key_properties {
        #[allow(unused_mut)]
        let mut object_4 = object.key("RightJoinKeyProperties").start_object();
        crate::protocol_serde::shape_join_key_properties::ser_join_key_properties(&mut object_4, var_3)?;
        object_4.finish();
    }
    {
        object.key("Type").string(input.r#type.as_str());
    }
    {
        object.key("OnClause").string(input.on_clause.as_str());
    }
    Ok(())
}

pub(crate) fn de_join_instruction<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::JoinInstruction>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::JoinInstructionBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "LeftOperand" => {
                            builder = builder.set_left_operand(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "RightOperand" => {
                            builder = builder.set_right_operand(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "LeftJoinKeyProperties" => {
                            builder = builder
                                .set_left_join_key_properties(crate::protocol_serde::shape_join_key_properties::de_join_key_properties(tokens)?);
                        }
                        "RightJoinKeyProperties" => {
                            builder = builder
                                .set_right_join_key_properties(crate::protocol_serde::shape_join_key_properties::de_join_key_properties(tokens)?);
                        }
                        "Type" => {
                            builder = builder.set_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::JoinType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "OnClause" => {
                            builder = builder.set_on_clause(
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
            Ok(Some(crate::serde_util::join_instruction_correct_errors(builder).build().map_err(
                |err| ::aws_smithy_json::deserialize::error::DeserializeError::custom_source("Response was invalid", err),
            )?))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

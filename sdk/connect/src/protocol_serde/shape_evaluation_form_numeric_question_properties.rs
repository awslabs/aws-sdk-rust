// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_evaluation_form_numeric_question_properties(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::EvaluationFormNumericQuestionProperties,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("MinValue").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.min_value).into()),
        );
    }
    {
        object.key("MaxValue").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.max_value).into()),
        );
    }
    if let Some(var_1) = &input.options {
        let mut array_2 = object.key("Options").start_array();
        for item_3 in var_1 {
            {
                #[allow(unused_mut)]
                let mut object_4 = array_2.value().start_object();
                crate::protocol_serde::shape_evaluation_form_numeric_question_option::ser_evaluation_form_numeric_question_option(
                    &mut object_4,
                    item_3,
                )?;
                object_4.finish();
            }
        }
        array_2.finish();
    }
    if let Some(var_5) = &input.automation {
        #[allow(unused_mut)]
        let mut object_6 = object.key("Automation").start_object();
        crate::protocol_serde::shape_evaluation_form_numeric_question_automation::ser_evaluation_form_numeric_question_automation(
            &mut object_6,
            var_5,
        )?;
        object_6.finish();
    }
    Ok(())
}

pub(crate) fn de_evaluation_form_numeric_question_properties<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::EvaluationFormNumericQuestionProperties>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::EvaluationFormNumericQuestionPropertiesBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "MinValue" => {
                            builder = builder.set_min_value(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "MaxValue" => {
                            builder = builder.set_max_value(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "Options" => {
                            builder = builder.set_options(
                                    crate::protocol_serde::shape_evaluation_form_numeric_question_option_list::de_evaluation_form_numeric_question_option_list(tokens)?
                                );
                        }
                        "Automation" => {
                            builder = builder.set_automation(
                                    crate::protocol_serde::shape_evaluation_form_numeric_question_automation::de_evaluation_form_numeric_question_automation(tokens)?
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
            Ok(Some(
                crate::serde_util::evaluation_form_numeric_question_properties_correct_errors(builder).build(),
            ))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

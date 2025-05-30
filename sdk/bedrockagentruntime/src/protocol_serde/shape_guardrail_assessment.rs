// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_guardrail_assessment<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::GuardrailAssessment>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::GuardrailAssessmentBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "topicPolicy" => {
                            builder = builder.set_topic_policy(
                                crate::protocol_serde::shape_guardrail_topic_policy_assessment::de_guardrail_topic_policy_assessment(tokens)?,
                            );
                        }
                        "contentPolicy" => {
                            builder = builder.set_content_policy(
                                crate::protocol_serde::shape_guardrail_content_policy_assessment::de_guardrail_content_policy_assessment(tokens)?,
                            );
                        }
                        "wordPolicy" => {
                            builder = builder.set_word_policy(
                                crate::protocol_serde::shape_guardrail_word_policy_assessment::de_guardrail_word_policy_assessment(tokens)?,
                            );
                        }
                        "sensitiveInformationPolicy" => {
                            builder = builder.set_sensitive_information_policy(
                                    crate::protocol_serde::shape_guardrail_sensitive_information_policy_assessment::de_guardrail_sensitive_information_policy_assessment(tokens)?
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

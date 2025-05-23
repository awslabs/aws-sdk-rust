// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_rightsizing_recommendation<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::RightsizingRecommendation>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::RightsizingRecommendationBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "AccountId" => {
                            builder = builder.set_account_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "CurrentInstance" => {
                            builder = builder.set_current_instance(crate::protocol_serde::shape_current_instance::de_current_instance(tokens)?);
                        }
                        "RightsizingType" => {
                            builder = builder.set_rightsizing_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::RightsizingType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "ModifyRecommendationDetail" => {
                            builder = builder.set_modify_recommendation_detail(
                                crate::protocol_serde::shape_modify_recommendation_detail::de_modify_recommendation_detail(tokens)?,
                            );
                        }
                        "TerminateRecommendationDetail" => {
                            builder = builder.set_terminate_recommendation_detail(
                                crate::protocol_serde::shape_terminate_recommendation_detail::de_terminate_recommendation_detail(tokens)?,
                            );
                        }
                        "FindingReasonCodes" => {
                            builder =
                                builder.set_finding_reason_codes(crate::protocol_serde::shape_finding_reason_codes::de_finding_reason_codes(tokens)?);
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

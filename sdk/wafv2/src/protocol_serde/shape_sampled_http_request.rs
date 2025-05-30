// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_sampled_http_request<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::SampledHttpRequest>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::SampledHttpRequestBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "Request" => {
                            builder = builder.set_request(crate::protocol_serde::shape_http_request::de_http_request(tokens)?);
                        }
                        "Weight" => {
                            builder = builder.set_weight(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "Timestamp" => {
                            builder = builder.set_timestamp(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "Action" => {
                            builder = builder.set_action(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "RuleNameWithinRuleGroup" => {
                            builder = builder.set_rule_name_within_rule_group(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "RequestHeadersInserted" => {
                            builder = builder.set_request_headers_inserted(crate::protocol_serde::shape_http_headers::de_http_headers(tokens)?);
                        }
                        "ResponseCodeSent" => {
                            builder = builder.set_response_code_sent(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "Labels" => {
                            builder = builder.set_labels(crate::protocol_serde::shape_labels::de_labels(tokens)?);
                        }
                        "CaptchaResponse" => {
                            builder = builder.set_captcha_response(crate::protocol_serde::shape_captcha_response::de_captcha_response(tokens)?);
                        }
                        "ChallengeResponse" => {
                            builder = builder.set_challenge_response(crate::protocol_serde::shape_challenge_response::de_challenge_response(tokens)?);
                        }
                        "OverriddenAction" => {
                            builder = builder.set_overridden_action(
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
            Ok(Some(crate::serde_util::sampled_http_request_correct_errors(builder).build()))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

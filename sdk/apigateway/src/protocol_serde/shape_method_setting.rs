// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_method_setting<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::MethodSetting>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::MethodSettingBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "metricsEnabled" => {
                            builder = builder.set_metrics_enabled(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "loggingLevel" => {
                            builder = builder.set_logging_level(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "dataTraceEnabled" => {
                            builder = builder.set_data_trace_enabled(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "throttlingBurstLimit" => {
                            builder = builder.set_throttling_burst_limit(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "throttlingRateLimit" => {
                            builder = builder.set_throttling_rate_limit(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?.map(|v| v.to_f64_lossy()),
                            );
                        }
                        "cachingEnabled" => {
                            builder = builder.set_caching_enabled(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "cacheTtlInSeconds" => {
                            builder = builder.set_cache_ttl_in_seconds(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "cacheDataEncrypted" => {
                            builder = builder.set_cache_data_encrypted(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "requireAuthorizationForCacheControl" => {
                            builder = builder.set_require_authorization_for_cache_control(
                                ::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?,
                            );
                        }
                        "unauthorizedCacheControlHeaderStrategy" => {
                            builder = builder.set_unauthorized_cache_control_header_strategy(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| {
                                        s.to_unescaped()
                                            .map(|u| crate::types::UnauthorizedCacheControlHeaderStrategy::from(u.as_ref()))
                                    })
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

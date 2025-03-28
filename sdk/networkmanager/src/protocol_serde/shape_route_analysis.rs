// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_route_analysis<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::RouteAnalysis>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::RouteAnalysisBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "GlobalNetworkId" => {
                            builder = builder.set_global_network_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "OwnerAccountId" => {
                            builder = builder.set_owner_account_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "RouteAnalysisId" => {
                            builder = builder.set_route_analysis_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "StartTimestamp" => {
                            builder = builder.set_start_timestamp(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "Status" => {
                            builder = builder.set_status(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::RouteAnalysisStatus::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "Source" => {
                            builder = builder.set_source(
                                crate::protocol_serde::shape_route_analysis_endpoint_options::de_route_analysis_endpoint_options(tokens)?,
                            );
                        }
                        "Destination" => {
                            builder = builder.set_destination(
                                crate::protocol_serde::shape_route_analysis_endpoint_options::de_route_analysis_endpoint_options(tokens)?,
                            );
                        }
                        "IncludeReturnPath" => {
                            builder = builder.set_include_return_path(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "UseMiddleboxes" => {
                            builder = builder.set_use_middleboxes(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "ForwardPath" => {
                            builder = builder.set_forward_path(crate::protocol_serde::shape_route_analysis_path::de_route_analysis_path(tokens)?);
                        }
                        "ReturnPath" => {
                            builder = builder.set_return_path(crate::protocol_serde::shape_route_analysis_path::de_route_analysis_path(tokens)?);
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

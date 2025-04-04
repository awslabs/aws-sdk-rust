// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_finding_statistics<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::FindingStatistics>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::FindingStatisticsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "countBySeverity" => {
                            builder = builder.set_count_by_severity(crate::protocol_serde::shape_count_by_severity::de_count_by_severity(tokens)?);
                        }
                        "groupedByAccount" => {
                            builder = builder.set_grouped_by_account(crate::protocol_serde::shape_grouped_by_account::de_grouped_by_account(tokens)?);
                        }
                        "groupedByDate" => {
                            builder = builder.set_grouped_by_date(crate::protocol_serde::shape_grouped_by_date::de_grouped_by_date(tokens)?);
                        }
                        "groupedByFindingType" => {
                            builder = builder.set_grouped_by_finding_type(
                                crate::protocol_serde::shape_grouped_by_finding_type::de_grouped_by_finding_type(tokens)?,
                            );
                        }
                        "groupedByResource" => {
                            builder =
                                builder.set_grouped_by_resource(crate::protocol_serde::shape_grouped_by_resource::de_grouped_by_resource(tokens)?);
                        }
                        "groupedBySeverity" => {
                            builder =
                                builder.set_grouped_by_severity(crate::protocol_serde::shape_grouped_by_severity::de_grouped_by_severity(tokens)?);
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

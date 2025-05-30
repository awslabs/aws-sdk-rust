// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_time_range_drill_down_filter(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::TimeRangeDrillDownFilter,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.column {
        #[allow(unused_mut)]
        let mut object_2 = object.key("Column").start_object();
        crate::protocol_serde::shape_column_identifier::ser_column_identifier(&mut object_2, var_1)?;
        object_2.finish();
    }
    {
        object
            .key("RangeMinimum")
            .date_time(&input.range_minimum, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    {
        object
            .key("RangeMaximum")
            .date_time(&input.range_maximum, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    {
        object.key("TimeGranularity").string(input.time_granularity.as_str());
    }
    Ok(())
}

pub(crate) fn de_time_range_drill_down_filter<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::TimeRangeDrillDownFilter>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::TimeRangeDrillDownFilterBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "Column" => {
                            builder = builder.set_column(crate::protocol_serde::shape_column_identifier::de_column_identifier(tokens)?);
                        }
                        "RangeMinimum" => {
                            builder = builder.set_range_minimum(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "RangeMaximum" => {
                            builder = builder.set_range_maximum(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                                tokens.next(),
                                ::aws_smithy_types::date_time::Format::EpochSeconds,
                            )?);
                        }
                        "TimeGranularity" => {
                            builder = builder.set_time_granularity(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::TimeGranularity::from(u.as_ref())))
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
            Ok(Some(
                crate::serde_util::time_range_drill_down_filter_correct_errors(builder)
                    .build()
                    .map_err(|err| ::aws_smithy_json::deserialize::error::DeserializeError::custom_source("Response was invalid", err))?,
            ))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

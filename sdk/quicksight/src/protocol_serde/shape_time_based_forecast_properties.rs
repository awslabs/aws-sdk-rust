// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_time_based_forecast_properties(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::TimeBasedForecastProperties,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.periods_forward {
        object.key("PeriodsForward").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_1).into()),
        );
    }
    if let Some(var_2) = &input.periods_backward {
        object.key("PeriodsBackward").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_2).into()),
        );
    }
    if let Some(var_3) = &input.upper_boundary {
        object.key("UpperBoundary").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::Float((*var_3).into()),
        );
    }
    if let Some(var_4) = &input.lower_boundary {
        object.key("LowerBoundary").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::Float((*var_4).into()),
        );
    }
    if let Some(var_5) = &input.prediction_interval {
        object.key("PredictionInterval").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_5).into()),
        );
    }
    if let Some(var_6) = &input.seasonality {
        object.key("Seasonality").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_6).into()),
        );
    }
    Ok(())
}

pub(crate) fn de_time_based_forecast_properties<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::TimeBasedForecastProperties>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::TimeBasedForecastPropertiesBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "PeriodsForward" => {
                            builder = builder.set_periods_forward(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "PeriodsBackward" => {
                            builder = builder.set_periods_backward(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "UpperBoundary" => {
                            builder = builder.set_upper_boundary(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?.map(|v| v.to_f64_lossy()),
                            );
                        }
                        "LowerBoundary" => {
                            builder = builder.set_lower_boundary(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?.map(|v| v.to_f64_lossy()),
                            );
                        }
                        "PredictionInterval" => {
                            builder = builder.set_prediction_interval(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "Seasonality" => {
                            builder = builder.set_seasonality(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
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

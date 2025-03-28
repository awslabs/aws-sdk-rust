// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_line_chart_default_series_settings(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::LineChartDefaultSeriesSettings,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.axis_binding {
        object.key("AxisBinding").string(var_1.as_str());
    }
    if let Some(var_2) = &input.line_style_settings {
        #[allow(unused_mut)]
        let mut object_3 = object.key("LineStyleSettings").start_object();
        crate::protocol_serde::shape_line_chart_line_style_settings::ser_line_chart_line_style_settings(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.marker_style_settings {
        #[allow(unused_mut)]
        let mut object_5 = object.key("MarkerStyleSettings").start_object();
        crate::protocol_serde::shape_line_chart_marker_style_settings::ser_line_chart_marker_style_settings(&mut object_5, var_4)?;
        object_5.finish();
    }
    Ok(())
}

pub(crate) fn de_line_chart_default_series_settings<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::LineChartDefaultSeriesSettings>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::LineChartDefaultSeriesSettingsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "AxisBinding" => {
                            builder = builder.set_axis_binding(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::AxisBinding::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "LineStyleSettings" => {
                            builder = builder.set_line_style_settings(
                                crate::protocol_serde::shape_line_chart_line_style_settings::de_line_chart_line_style_settings(tokens)?,
                            );
                        }
                        "MarkerStyleSettings" => {
                            builder = builder.set_marker_style_settings(
                                crate::protocol_serde::shape_line_chart_marker_style_settings::de_line_chart_marker_style_settings(tokens)?,
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

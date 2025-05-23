// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_parametric_configuration_overrides(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ParametricConfigurationOverrides,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.application_configuration {
        let mut array_2 = object.key("applicationConfiguration").start_array();
        for item_3 in var_1 {
            {
                #[allow(unused_mut)]
                let mut object_4 = array_2.value().start_object();
                crate::protocol_serde::shape_configuration::ser_configuration(&mut object_4, item_3)?;
                object_4.finish();
            }
        }
        array_2.finish();
    }
    if let Some(var_5) = &input.monitoring_configuration {
        #[allow(unused_mut)]
        let mut object_6 = object.key("monitoringConfiguration").start_object();
        crate::protocol_serde::shape_parametric_monitoring_configuration::ser_parametric_monitoring_configuration(&mut object_6, var_5)?;
        object_6.finish();
    }
    Ok(())
}

pub(crate) fn de_parametric_configuration_overrides<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::ParametricConfigurationOverrides>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::ParametricConfigurationOverridesBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "applicationConfiguration" => {
                            builder = builder
                                .set_application_configuration(crate::protocol_serde::shape_configuration_list::de_configuration_list(tokens)?);
                        }
                        "monitoringConfiguration" => {
                            builder = builder.set_monitoring_configuration(
                                crate::protocol_serde::shape_parametric_monitoring_configuration::de_parametric_monitoring_configuration(tokens)?,
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

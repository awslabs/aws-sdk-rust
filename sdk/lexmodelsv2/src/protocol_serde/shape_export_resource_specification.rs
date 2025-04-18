// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_export_resource_specification<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::ExportResourceSpecification>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::ExportResourceSpecificationBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "botExportSpecification" => {
                            builder = builder.set_bot_export_specification(
                                crate::protocol_serde::shape_bot_export_specification::de_bot_export_specification(tokens)?,
                            );
                        }
                        "botLocaleExportSpecification" => {
                            builder = builder.set_bot_locale_export_specification(
                                crate::protocol_serde::shape_bot_locale_export_specification::de_bot_locale_export_specification(tokens)?,
                            );
                        }
                        "customVocabularyExportSpecification" => {
                            builder = builder.set_custom_vocabulary_export_specification(
                                crate::protocol_serde::shape_custom_vocabulary_export_specification::de_custom_vocabulary_export_specification(
                                    tokens,
                                )?,
                            );
                        }
                        "testSetExportSpecification" => {
                            builder = builder.set_test_set_export_specification(
                                crate::protocol_serde::shape_test_set_export_specification::de_test_set_export_specification(tokens)?,
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

pub fn ser_export_resource_specification(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ExportResourceSpecification,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.bot_export_specification {
        #[allow(unused_mut)]
        let mut object_2 = object.key("botExportSpecification").start_object();
        crate::protocol_serde::shape_bot_export_specification::ser_bot_export_specification(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.bot_locale_export_specification {
        #[allow(unused_mut)]
        let mut object_4 = object.key("botLocaleExportSpecification").start_object();
        crate::protocol_serde::shape_bot_locale_export_specification::ser_bot_locale_export_specification(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.custom_vocabulary_export_specification {
        #[allow(unused_mut)]
        let mut object_6 = object.key("customVocabularyExportSpecification").start_object();
        crate::protocol_serde::shape_custom_vocabulary_export_specification::ser_custom_vocabulary_export_specification(&mut object_6, var_5)?;
        object_6.finish();
    }
    if let Some(var_7) = &input.test_set_export_specification {
        #[allow(unused_mut)]
        let mut object_8 = object.key("testSetExportSpecification").start_object();
        crate::protocol_serde::shape_test_set_export_specification::ser_test_set_export_specification(&mut object_8, var_7)?;
        object_8.finish();
    }
    Ok(())
}

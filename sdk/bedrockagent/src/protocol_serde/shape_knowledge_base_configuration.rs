// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_knowledge_base_configuration(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::KnowledgeBaseConfiguration,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("type").string(input.r#type.as_str());
    }
    if let Some(var_1) = &input.vector_knowledge_base_configuration {
        #[allow(unused_mut)]
        let mut object_2 = object.key("vectorKnowledgeBaseConfiguration").start_object();
        crate::protocol_serde::shape_vector_knowledge_base_configuration::ser_vector_knowledge_base_configuration(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.kendra_knowledge_base_configuration {
        #[allow(unused_mut)]
        let mut object_4 = object.key("kendraKnowledgeBaseConfiguration").start_object();
        crate::protocol_serde::shape_kendra_knowledge_base_configuration::ser_kendra_knowledge_base_configuration(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.sql_knowledge_base_configuration {
        #[allow(unused_mut)]
        let mut object_6 = object.key("sqlKnowledgeBaseConfiguration").start_object();
        crate::protocol_serde::shape_sql_knowledge_base_configuration::ser_sql_knowledge_base_configuration(&mut object_6, var_5)?;
        object_6.finish();
    }
    Ok(())
}

pub(crate) fn de_knowledge_base_configuration<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::KnowledgeBaseConfiguration>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::KnowledgeBaseConfigurationBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "type" => {
                            builder = builder.set_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::KnowledgeBaseType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "vectorKnowledgeBaseConfiguration" => {
                            builder = builder.set_vector_knowledge_base_configuration(
                                crate::protocol_serde::shape_vector_knowledge_base_configuration::de_vector_knowledge_base_configuration(tokens)?,
                            );
                        }
                        "kendraKnowledgeBaseConfiguration" => {
                            builder = builder.set_kendra_knowledge_base_configuration(
                                crate::protocol_serde::shape_kendra_knowledge_base_configuration::de_kendra_knowledge_base_configuration(tokens)?,
                            );
                        }
                        "sqlKnowledgeBaseConfiguration" => {
                            builder = builder.set_sql_knowledge_base_configuration(
                                crate::protocol_serde::shape_sql_knowledge_base_configuration::de_sql_knowledge_base_configuration(tokens)?,
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
                crate::serde_util::knowledge_base_configuration_correct_errors(builder)
                    .build()
                    .map_err(|err| ::aws_smithy_json::deserialize::error::DeserializeError::custom_source("Response was invalid", err))?,
            ))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_label_schema(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::LabelSchema,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.label_mapper {
        #[allow(unused_mut)]
        let mut object_2 = object.key("labelMapper").start_object();
        for (key_3, value_4) in var_1 {
            {
                let mut array_5 = object_2.key(key_3.as_str()).start_array();
                for item_6 in value_4 {
                    {
                        array_5.value().string(item_6.as_str());
                    }
                }
                array_5.finish();
            }
        }
        object_2.finish();
    }
    if let Some(var_7) = &input.unlabeled_events_treatment {
        object.key("unlabeledEventsTreatment").string(var_7.as_str());
    }
    Ok(())
}

pub(crate) fn de_label_schema<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::LabelSchema>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::LabelSchemaBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "labelMapper" => {
                            builder = builder.set_label_mapper(crate::protocol_serde::shape_label_mapper::de_label_mapper(tokens)?);
                        }
                        "unlabeledEventsTreatment" => {
                            builder = builder.set_unlabeled_events_treatment(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::UnlabeledEventsTreatment::from(u.as_ref())))
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

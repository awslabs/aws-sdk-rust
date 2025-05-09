// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_event_included_data<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::EventIncludedData>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::EventIncludedDataBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "caseData" => {
                            builder = builder.set_case_data(crate::protocol_serde::shape_case_event_included_data::de_case_event_included_data(
                                tokens,
                            )?);
                        }
                        "relatedItemData" => {
                            builder = builder.set_related_item_data(
                                crate::protocol_serde::shape_related_item_event_included_data::de_related_item_event_included_data(tokens)?,
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

pub fn ser_event_included_data(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::EventIncludedData,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.case_data {
        #[allow(unused_mut)]
        let mut object_2 = object.key("caseData").start_object();
        crate::protocol_serde::shape_case_event_included_data::ser_case_event_included_data(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.related_item_data {
        #[allow(unused_mut)]
        let mut object_4 = object.key("relatedItemData").start_object();
        crate::protocol_serde::shape_related_item_event_included_data::ser_related_item_event_included_data(&mut object_4, var_3)?;
        object_4.finish();
    }
    Ok(())
}

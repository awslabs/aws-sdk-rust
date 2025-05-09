// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_coverage_time(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::CoverageTime,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.start {
        #[allow(unused_mut)]
        let mut object_2 = object.key("Start").start_object();
        crate::protocol_serde::shape_hand_off_time::ser_hand_off_time(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.end {
        #[allow(unused_mut)]
        let mut object_4 = object.key("End").start_object();
        crate::protocol_serde::shape_hand_off_time::ser_hand_off_time(&mut object_4, var_3)?;
        object_4.finish();
    }
    Ok(())
}

pub(crate) fn de_coverage_time<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::CoverageTime>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::CoverageTimeBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "Start" => {
                            builder = builder.set_start(crate::protocol_serde::shape_hand_off_time::de_hand_off_time(tokens)?);
                        }
                        "End" => {
                            builder = builder.set_end(crate::protocol_serde::shape_hand_off_time::de_hand_off_time(tokens)?);
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

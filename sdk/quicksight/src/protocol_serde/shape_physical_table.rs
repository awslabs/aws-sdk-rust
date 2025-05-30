// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_physical_table(
    object_41: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::PhysicalTable,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    match input {
        crate::types::PhysicalTable::RelationalTable(inner) => {
            #[allow(unused_mut)]
            let mut object_1 = object_41.key("RelationalTable").start_object();
            crate::protocol_serde::shape_relational_table::ser_relational_table(&mut object_1, inner)?;
            object_1.finish();
        }
        crate::types::PhysicalTable::CustomSql(inner) => {
            #[allow(unused_mut)]
            let mut object_2 = object_41.key("CustomSql").start_object();
            crate::protocol_serde::shape_custom_sql::ser_custom_sql(&mut object_2, inner)?;
            object_2.finish();
        }
        crate::types::PhysicalTable::S3Source(inner) => {
            #[allow(unused_mut)]
            let mut object_3 = object_41.key("S3Source").start_object();
            crate::protocol_serde::shape_s3_source::ser_s3_source(&mut object_3, inner)?;
            object_3.finish();
        }
        crate::types::PhysicalTable::Unknown => {
            return Err(::aws_smithy_types::error::operation::SerializationError::unknown_variant("PhysicalTable"))
        }
    }
    Ok(())
}

pub(crate) fn de_physical_table<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::PhysicalTable>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    let mut variant = None;
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => return Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => loop {
            match tokens.next().transpose()? {
                Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => {
                    if let ::std::option::Option::Some(::std::result::Result::Ok(::aws_smithy_json::deserialize::Token::ValueNull { .. })) =
                        tokens.peek()
                    {
                        let _ = tokens.next().expect("peek returned a token")?;
                        continue;
                    }
                    let key = key.to_unescaped()?;
                    if key == "__type" {
                        ::aws_smithy_json::deserialize::token::skip_value(tokens)?;
                        continue;
                    }
                    if variant.is_some() {
                        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
                            "encountered mixed variants in union",
                        ));
                    }
                    variant = match key.as_ref() {
                        "RelationalTable" => Some(crate::types::PhysicalTable::RelationalTable(
                            crate::protocol_serde::shape_relational_table::de_relational_table(tokens)?.ok_or_else(|| {
                                ::aws_smithy_json::deserialize::error::DeserializeError::custom("value for 'RelationalTable' cannot be null")
                            })?,
                        )),
                        "CustomSql" => Some(crate::types::PhysicalTable::CustomSql(
                            crate::protocol_serde::shape_custom_sql::de_custom_sql(tokens)?.ok_or_else(|| {
                                ::aws_smithy_json::deserialize::error::DeserializeError::custom("value for 'CustomSql' cannot be null")
                            })?,
                        )),
                        "S3Source" => Some(crate::types::PhysicalTable::S3Source(
                            crate::protocol_serde::shape_s3_source::de_s3_source(tokens)?.ok_or_else(|| {
                                ::aws_smithy_json::deserialize::error::DeserializeError::custom("value for 'S3Source' cannot be null")
                            })?,
                        )),
                        _ => {
                            ::aws_smithy_json::deserialize::token::skip_value(tokens)?;
                            Some(crate::types::PhysicalTable::Unknown)
                        }
                    };
                }
                other => {
                    return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(format!(
                        "expected object key or end object, found: {:?}",
                        other
                    )))
                }
            }
        },
        _ => {
            return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
                "expected start object or null",
            ))
        }
    }
    if variant.is_none() {
        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "Union did not contain a valid variant.",
        ));
    }
    Ok(variant)
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_schema_attribute_type(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::SchemaAttributeType,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.name {
        object.key("Name").string(var_1.as_str());
    }
    if let Some(var_2) = &input.attribute_data_type {
        object.key("AttributeDataType").string(var_2.as_str());
    }
    if let Some(var_3) = &input.developer_only_attribute {
        object.key("DeveloperOnlyAttribute").boolean(*var_3);
    }
    if let Some(var_4) = &input.mutable {
        object.key("Mutable").boolean(*var_4);
    }
    if let Some(var_5) = &input.required {
        object.key("Required").boolean(*var_5);
    }
    if let Some(var_6) = &input.number_attribute_constraints {
        #[allow(unused_mut)]
        let mut object_7 = object.key("NumberAttributeConstraints").start_object();
        crate::protocol_serde::shape_number_attribute_constraints_type::ser_number_attribute_constraints_type(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.string_attribute_constraints {
        #[allow(unused_mut)]
        let mut object_9 = object.key("StringAttributeConstraints").start_object();
        crate::protocol_serde::shape_string_attribute_constraints_type::ser_string_attribute_constraints_type(&mut object_9, var_8)?;
        object_9.finish();
    }
    Ok(())
}

pub(crate) fn de_schema_attribute_type<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::SchemaAttributeType>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::SchemaAttributeTypeBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "Name" => {
                            builder = builder.set_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "AttributeDataType" => {
                            builder = builder.set_attribute_data_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::AttributeDataType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "DeveloperOnlyAttribute" => {
                            builder =
                                builder.set_developer_only_attribute(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "Mutable" => {
                            builder = builder.set_mutable(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "Required" => {
                            builder = builder.set_required(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "NumberAttributeConstraints" => {
                            builder = builder.set_number_attribute_constraints(
                                crate::protocol_serde::shape_number_attribute_constraints_type::de_number_attribute_constraints_type(tokens)?,
                            );
                        }
                        "StringAttributeConstraints" => {
                            builder = builder.set_string_attribute_constraints(
                                crate::protocol_serde::shape_string_attribute_constraints_type::de_string_attribute_constraints_type(tokens)?,
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

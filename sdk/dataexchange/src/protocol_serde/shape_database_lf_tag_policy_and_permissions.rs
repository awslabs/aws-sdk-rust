// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_database_lf_tag_policy_and_permissions(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::DatabaseLfTagPolicyAndPermissions,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        let mut array_1 = object.key("Expression").start_array();
        for item_2 in &input.expression {
            {
                #[allow(unused_mut)]
                let mut object_3 = array_1.value().start_object();
                crate::protocol_serde::shape_lf_tag::ser_lf_tag(&mut object_3, item_2)?;
                object_3.finish();
            }
        }
        array_1.finish();
    }
    {
        let mut array_4 = object.key("Permissions").start_array();
        for item_5 in &input.permissions {
            {
                array_4.value().string(item_5.as_str());
            }
        }
        array_4.finish();
    }
    Ok(())
}

pub(crate) fn de_database_lf_tag_policy_and_permissions<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::DatabaseLfTagPolicyAndPermissions>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::DatabaseLfTagPolicyAndPermissionsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "Expression" => {
                            builder = builder.set_expression(crate::protocol_serde::shape_list_of_lf_tags::de_list_of_lf_tags(tokens)?);
                        }
                        "Permissions" => {
                            builder = builder.set_permissions(
                                    crate::protocol_serde::shape_list_of_database_lf_tag_policy_permissions::de_list_of_database_lf_tag_policy_permissions(tokens)?
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
                crate::serde_util::database_lf_tag_policy_and_permissions_correct_errors(builder)
                    .build()
                    .map_err(|err| ::aws_smithy_json::deserialize::error::DeserializeError::custom_source("Response was invalid", err))?,
            ))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

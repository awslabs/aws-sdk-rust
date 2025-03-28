// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_test_set_import_resource_specification<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::TestSetImportResourceSpecification>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::TestSetImportResourceSpecificationBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "testSetName" => {
                            builder = builder.set_test_set_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "description" => {
                            builder = builder.set_description(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "roleArn" => {
                            builder = builder.set_role_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "storageLocation" => {
                            builder = builder.set_storage_location(
                                crate::protocol_serde::shape_test_set_storage_location::de_test_set_storage_location(tokens)?,
                            );
                        }
                        "importInputLocation" => {
                            builder = builder.set_import_input_location(
                                crate::protocol_serde::shape_test_set_import_input_location::de_test_set_import_input_location(tokens)?,
                            );
                        }
                        "modality" => {
                            builder = builder.set_modality(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::TestSetModality::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "testSetTags" => {
                            builder = builder.set_test_set_tags(crate::protocol_serde::shape_tag_map::de_tag_map(tokens)?);
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
                crate::serde_util::test_set_import_resource_specification_correct_errors(builder)
                    .build()
                    .map_err(|err| ::aws_smithy_json::deserialize::error::DeserializeError::custom_source("Response was invalid", err))?,
            ))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

pub fn ser_test_set_import_resource_specification(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::TestSetImportResourceSpecification,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("testSetName").string(input.test_set_name.as_str());
    }
    if let Some(var_1) = &input.description {
        object.key("description").string(var_1.as_str());
    }
    {
        object.key("roleArn").string(input.role_arn.as_str());
    }
    if let Some(var_2) = &input.storage_location {
        #[allow(unused_mut)]
        let mut object_3 = object.key("storageLocation").start_object();
        crate::protocol_serde::shape_test_set_storage_location::ser_test_set_storage_location(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.import_input_location {
        #[allow(unused_mut)]
        let mut object_5 = object.key("importInputLocation").start_object();
        crate::protocol_serde::shape_test_set_import_input_location::ser_test_set_import_input_location(&mut object_5, var_4)?;
        object_5.finish();
    }
    {
        object.key("modality").string(input.modality.as_str());
    }
    if let Some(var_6) = &input.test_set_tags {
        #[allow(unused_mut)]
        let mut object_7 = object.key("testSetTags").start_object();
        for (key_8, value_9) in var_6 {
            {
                object_7.key(key_8.as_str()).string(value_9.as_str());
            }
        }
        object_7.finish();
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_sapo_data_destination_properties(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::SapoDataDestinationProperties,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("objectPath").string(input.object_path.as_str());
    }
    if let Some(var_1) = &input.success_response_handling_config {
        #[allow(unused_mut)]
        let mut object_2 = object.key("successResponseHandlingConfig").start_object();
        crate::protocol_serde::shape_success_response_handling_config::ser_success_response_handling_config(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.id_field_names {
        let mut array_4 = object.key("idFieldNames").start_array();
        for item_5 in var_3 {
            {
                array_4.value().string(item_5.as_str());
            }
        }
        array_4.finish();
    }
    if let Some(var_6) = &input.error_handling_config {
        #[allow(unused_mut)]
        let mut object_7 = object.key("errorHandlingConfig").start_object();
        crate::protocol_serde::shape_error_handling_config::ser_error_handling_config(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.write_operation_type {
        object.key("writeOperationType").string(var_8.as_str());
    }
    Ok(())
}

pub(crate) fn de_sapo_data_destination_properties<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::SapoDataDestinationProperties>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::SapoDataDestinationPropertiesBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "objectPath" => {
                            builder = builder.set_object_path(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "successResponseHandlingConfig" => {
                            builder = builder.set_success_response_handling_config(
                                crate::protocol_serde::shape_success_response_handling_config::de_success_response_handling_config(tokens)?,
                            );
                        }
                        "idFieldNames" => {
                            builder = builder.set_id_field_names(crate::protocol_serde::shape_id_field_name_list::de_id_field_name_list(tokens)?);
                        }
                        "errorHandlingConfig" => {
                            builder = builder
                                .set_error_handling_config(crate::protocol_serde::shape_error_handling_config::de_error_handling_config(tokens)?);
                        }
                        "writeOperationType" => {
                            builder = builder.set_write_operation_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::WriteOperationType::from(u.as_ref())))
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
            Ok(Some(
                crate::serde_util::sapo_data_destination_properties_correct_errors(builder)
                    .build()
                    .map_err(|err| ::aws_smithy_json::deserialize::error::DeserializeError::custom_source("Response was invalid", err))?,
            ))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

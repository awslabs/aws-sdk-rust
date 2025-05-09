// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_resource_mapping(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ResourceMapping,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.resource_name {
        object.key("resourceName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.logical_stack_name {
        object.key("logicalStackName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.app_registry_app_name {
        object.key("appRegistryAppName").string(var_3.as_str());
    }
    if let Some(var_4) = &input.resource_group_name {
        object.key("resourceGroupName").string(var_4.as_str());
    }
    {
        object.key("mappingType").string(input.mapping_type.as_str());
    }
    if let Some(var_5) = &input.physical_resource_id {
        #[allow(unused_mut)]
        let mut object_6 = object.key("physicalResourceId").start_object();
        crate::protocol_serde::shape_physical_resource_id::ser_physical_resource_id(&mut object_6, var_5)?;
        object_6.finish();
    }
    if let Some(var_7) = &input.terraform_source_name {
        object.key("terraformSourceName").string(var_7.as_str());
    }
    if let Some(var_8) = &input.eks_source_name {
        object.key("eksSourceName").string(var_8.as_str());
    }
    Ok(())
}

pub(crate) fn de_resource_mapping<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::ResourceMapping>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::ResourceMappingBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "resourceName" => {
                            builder = builder.set_resource_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "logicalStackName" => {
                            builder = builder.set_logical_stack_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "appRegistryAppName" => {
                            builder = builder.set_app_registry_app_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "resourceGroupName" => {
                            builder = builder.set_resource_group_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "mappingType" => {
                            builder = builder.set_mapping_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::ResourceMappingType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "physicalResourceId" => {
                            builder =
                                builder.set_physical_resource_id(crate::protocol_serde::shape_physical_resource_id::de_physical_resource_id(tokens)?);
                        }
                        "terraformSourceName" => {
                            builder = builder.set_terraform_source_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "eksSourceName" => {
                            builder = builder.set_eks_source_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
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
            Ok(Some(crate::serde_util::resource_mapping_correct_errors(builder).build().map_err(
                |err| ::aws_smithy_json::deserialize::error::DeserializeError::custom_source("Response was invalid", err),
            )?))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

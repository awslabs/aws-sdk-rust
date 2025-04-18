// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_eks_container(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::EksContainer,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.name {
        object.key("name").string(var_1.as_str());
    }
    if let Some(var_2) = &input.image {
        object.key("image").string(var_2.as_str());
    }
    if let Some(var_3) = &input.image_pull_policy {
        object.key("imagePullPolicy").string(var_3.as_str());
    }
    if let Some(var_4) = &input.command {
        let mut array_5 = object.key("command").start_array();
        for item_6 in var_4 {
            {
                array_5.value().string(item_6.as_str());
            }
        }
        array_5.finish();
    }
    if let Some(var_7) = &input.args {
        let mut array_8 = object.key("args").start_array();
        for item_9 in var_7 {
            {
                array_8.value().string(item_9.as_str());
            }
        }
        array_8.finish();
    }
    if let Some(var_10) = &input.env {
        let mut array_11 = object.key("env").start_array();
        for item_12 in var_10 {
            {
                #[allow(unused_mut)]
                let mut object_13 = array_11.value().start_object();
                crate::protocol_serde::shape_eks_container_environment_variable::ser_eks_container_environment_variable(&mut object_13, item_12)?;
                object_13.finish();
            }
        }
        array_11.finish();
    }
    if let Some(var_14) = &input.resources {
        #[allow(unused_mut)]
        let mut object_15 = object.key("resources").start_object();
        crate::protocol_serde::shape_eks_container_resource_requirements::ser_eks_container_resource_requirements(&mut object_15, var_14)?;
        object_15.finish();
    }
    if let Some(var_16) = &input.volume_mounts {
        let mut array_17 = object.key("volumeMounts").start_array();
        for item_18 in var_16 {
            {
                #[allow(unused_mut)]
                let mut object_19 = array_17.value().start_object();
                crate::protocol_serde::shape_eks_container_volume_mount::ser_eks_container_volume_mount(&mut object_19, item_18)?;
                object_19.finish();
            }
        }
        array_17.finish();
    }
    if let Some(var_20) = &input.security_context {
        #[allow(unused_mut)]
        let mut object_21 = object.key("securityContext").start_object();
        crate::protocol_serde::shape_eks_container_security_context::ser_eks_container_security_context(&mut object_21, var_20)?;
        object_21.finish();
    }
    Ok(())
}

pub(crate) fn de_eks_container<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::EksContainer>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::EksContainerBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "name" => {
                            builder = builder.set_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "image" => {
                            builder = builder.set_image(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "imagePullPolicy" => {
                            builder = builder.set_image_pull_policy(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "command" => {
                            builder = builder.set_command(crate::protocol_serde::shape_string_list::de_string_list(tokens)?);
                        }
                        "args" => {
                            builder = builder.set_args(crate::protocol_serde::shape_string_list::de_string_list(tokens)?);
                        }
                        "env" => {
                            builder = builder.set_env(
                                crate::protocol_serde::shape_eks_container_environment_variables::de_eks_container_environment_variables(tokens)?,
                            );
                        }
                        "resources" => {
                            builder = builder.set_resources(
                                crate::protocol_serde::shape_eks_container_resource_requirements::de_eks_container_resource_requirements(tokens)?,
                            );
                        }
                        "volumeMounts" => {
                            builder = builder.set_volume_mounts(
                                crate::protocol_serde::shape_eks_container_volume_mounts::de_eks_container_volume_mounts(tokens)?,
                            );
                        }
                        "securityContext" => {
                            builder = builder.set_security_context(
                                crate::protocol_serde::shape_eks_container_security_context::de_eks_container_security_context(tokens)?,
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
            Ok(Some(crate::serde_util::eks_container_correct_errors(builder).build()))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

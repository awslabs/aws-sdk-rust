// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_ecs_task_override(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::EcsTaskOverride,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.container_overrides {
        let mut array_2 = object.key("ContainerOverrides").start_array();
        for item_3 in var_1 {
            {
                #[allow(unused_mut)]
                let mut object_4 = array_2.value().start_object();
                crate::protocol_serde::shape_ecs_container_override::ser_ecs_container_override(&mut object_4, item_3)?;
                object_4.finish();
            }
        }
        array_2.finish();
    }
    if let Some(var_5) = &input.cpu {
        object.key("Cpu").string(var_5.as_str());
    }
    if let Some(var_6) = &input.ephemeral_storage {
        #[allow(unused_mut)]
        let mut object_7 = object.key("EphemeralStorage").start_object();
        crate::protocol_serde::shape_ecs_ephemeral_storage::ser_ecs_ephemeral_storage(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.execution_role_arn {
        object.key("ExecutionRoleArn").string(var_8.as_str());
    }
    if let Some(var_9) = &input.inference_accelerator_overrides {
        let mut array_10 = object.key("InferenceAcceleratorOverrides").start_array();
        for item_11 in var_9 {
            {
                #[allow(unused_mut)]
                let mut object_12 = array_10.value().start_object();
                crate::protocol_serde::shape_ecs_inference_accelerator_override::ser_ecs_inference_accelerator_override(&mut object_12, item_11)?;
                object_12.finish();
            }
        }
        array_10.finish();
    }
    if let Some(var_13) = &input.memory {
        object.key("Memory").string(var_13.as_str());
    }
    if let Some(var_14) = &input.task_role_arn {
        object.key("TaskRoleArn").string(var_14.as_str());
    }
    Ok(())
}

pub(crate) fn de_ecs_task_override<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::EcsTaskOverride>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::EcsTaskOverrideBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "ContainerOverrides" => {
                            builder = builder.set_container_overrides(
                                crate::protocol_serde::shape_ecs_container_override_list::de_ecs_container_override_list(tokens)?,
                            );
                        }
                        "Cpu" => {
                            builder = builder.set_cpu(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "EphemeralStorage" => {
                            builder =
                                builder.set_ephemeral_storage(crate::protocol_serde::shape_ecs_ephemeral_storage::de_ecs_ephemeral_storage(tokens)?);
                        }
                        "ExecutionRoleArn" => {
                            builder = builder.set_execution_role_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "InferenceAcceleratorOverrides" => {
                            builder = builder.set_inference_accelerator_overrides(
                                crate::protocol_serde::shape_ecs_inference_accelerator_override_list::de_ecs_inference_accelerator_override_list(
                                    tokens,
                                )?,
                            );
                        }
                        "Memory" => {
                            builder = builder.set_memory(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "TaskRoleArn" => {
                            builder = builder.set_task_role_arn(
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
            Ok(Some(builder.build()))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

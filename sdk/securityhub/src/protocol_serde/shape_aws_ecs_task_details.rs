// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_aws_ecs_task_details(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::AwsEcsTaskDetails,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.cluster_arn {
        object.key("ClusterArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.task_definition_arn {
        object.key("TaskDefinitionArn").string(var_2.as_str());
    }
    if let Some(var_3) = &input.version {
        object.key("Version").string(var_3.as_str());
    }
    if let Some(var_4) = &input.created_at {
        object.key("CreatedAt").string(var_4.as_str());
    }
    if let Some(var_5) = &input.started_at {
        object.key("StartedAt").string(var_5.as_str());
    }
    if let Some(var_6) = &input.started_by {
        object.key("StartedBy").string(var_6.as_str());
    }
    if let Some(var_7) = &input.group {
        object.key("Group").string(var_7.as_str());
    }
    if let Some(var_8) = &input.volumes {
        let mut array_9 = object.key("Volumes").start_array();
        for item_10 in var_8 {
            {
                #[allow(unused_mut)]
                let mut object_11 = array_9.value().start_object();
                crate::protocol_serde::shape_aws_ecs_task_volume_details::ser_aws_ecs_task_volume_details(&mut object_11, item_10)?;
                object_11.finish();
            }
        }
        array_9.finish();
    }
    if let Some(var_12) = &input.containers {
        let mut array_13 = object.key("Containers").start_array();
        for item_14 in var_12 {
            {
                #[allow(unused_mut)]
                let mut object_15 = array_13.value().start_object();
                crate::protocol_serde::shape_aws_ecs_container_details::ser_aws_ecs_container_details(&mut object_15, item_14)?;
                object_15.finish();
            }
        }
        array_13.finish();
    }
    Ok(())
}

pub(crate) fn de_aws_ecs_task_details<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::AwsEcsTaskDetails>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::AwsEcsTaskDetailsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "ClusterArn" => {
                            builder = builder.set_cluster_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "TaskDefinitionArn" => {
                            builder = builder.set_task_definition_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "Version" => {
                            builder = builder.set_version(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "CreatedAt" => {
                            builder = builder.set_created_at(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "StartedAt" => {
                            builder = builder.set_started_at(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "StartedBy" => {
                            builder = builder.set_started_by(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "Group" => {
                            builder = builder.set_group(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "Volumes" => {
                            builder = builder.set_volumes(
                                crate::protocol_serde::shape_aws_ecs_task_volume_details_list::de_aws_ecs_task_volume_details_list(tokens)?,
                            );
                        }
                        "Containers" => {
                            builder = builder.set_containers(
                                crate::protocol_serde::shape_aws_ecs_container_details_list::de_aws_ecs_container_details_list(tokens)?,
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

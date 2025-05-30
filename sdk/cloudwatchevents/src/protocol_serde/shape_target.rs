// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_target(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::Target,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("Id").string(input.id.as_str());
    }
    {
        object.key("Arn").string(input.arn.as_str());
    }
    if let Some(var_1) = &input.role_arn {
        object.key("RoleArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.input {
        object.key("Input").string(var_2.as_str());
    }
    if let Some(var_3) = &input.input_path {
        object.key("InputPath").string(var_3.as_str());
    }
    if let Some(var_4) = &input.input_transformer {
        #[allow(unused_mut)]
        let mut object_5 = object.key("InputTransformer").start_object();
        crate::protocol_serde::shape_input_transformer::ser_input_transformer(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.kinesis_parameters {
        #[allow(unused_mut)]
        let mut object_7 = object.key("KinesisParameters").start_object();
        crate::protocol_serde::shape_kinesis_parameters::ser_kinesis_parameters(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.run_command_parameters {
        #[allow(unused_mut)]
        let mut object_9 = object.key("RunCommandParameters").start_object();
        crate::protocol_serde::shape_run_command_parameters::ser_run_command_parameters(&mut object_9, var_8)?;
        object_9.finish();
    }
    if let Some(var_10) = &input.ecs_parameters {
        #[allow(unused_mut)]
        let mut object_11 = object.key("EcsParameters").start_object();
        crate::protocol_serde::shape_ecs_parameters::ser_ecs_parameters(&mut object_11, var_10)?;
        object_11.finish();
    }
    if let Some(var_12) = &input.batch_parameters {
        #[allow(unused_mut)]
        let mut object_13 = object.key("BatchParameters").start_object();
        crate::protocol_serde::shape_batch_parameters::ser_batch_parameters(&mut object_13, var_12)?;
        object_13.finish();
    }
    if let Some(var_14) = &input.sqs_parameters {
        #[allow(unused_mut)]
        let mut object_15 = object.key("SqsParameters").start_object();
        crate::protocol_serde::shape_sqs_parameters::ser_sqs_parameters(&mut object_15, var_14)?;
        object_15.finish();
    }
    if let Some(var_16) = &input.http_parameters {
        #[allow(unused_mut)]
        let mut object_17 = object.key("HttpParameters").start_object();
        crate::protocol_serde::shape_http_parameters::ser_http_parameters(&mut object_17, var_16)?;
        object_17.finish();
    }
    if let Some(var_18) = &input.redshift_data_parameters {
        #[allow(unused_mut)]
        let mut object_19 = object.key("RedshiftDataParameters").start_object();
        crate::protocol_serde::shape_redshift_data_parameters::ser_redshift_data_parameters(&mut object_19, var_18)?;
        object_19.finish();
    }
    if let Some(var_20) = &input.sage_maker_pipeline_parameters {
        #[allow(unused_mut)]
        let mut object_21 = object.key("SageMakerPipelineParameters").start_object();
        crate::protocol_serde::shape_sage_maker_pipeline_parameters::ser_sage_maker_pipeline_parameters(&mut object_21, var_20)?;
        object_21.finish();
    }
    if let Some(var_22) = &input.dead_letter_config {
        #[allow(unused_mut)]
        let mut object_23 = object.key("DeadLetterConfig").start_object();
        crate::protocol_serde::shape_dead_letter_config::ser_dead_letter_config(&mut object_23, var_22)?;
        object_23.finish();
    }
    if let Some(var_24) = &input.retry_policy {
        #[allow(unused_mut)]
        let mut object_25 = object.key("RetryPolicy").start_object();
        crate::protocol_serde::shape_retry_policy::ser_retry_policy(&mut object_25, var_24)?;
        object_25.finish();
    }
    Ok(())
}

pub(crate) fn de_target<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::Target>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::TargetBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "Id" => {
                            builder = builder.set_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "Arn" => {
                            builder = builder.set_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "RoleArn" => {
                            builder = builder.set_role_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "Input" => {
                            builder = builder.set_input(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "InputPath" => {
                            builder = builder.set_input_path(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "InputTransformer" => {
                            builder = builder.set_input_transformer(crate::protocol_serde::shape_input_transformer::de_input_transformer(tokens)?);
                        }
                        "KinesisParameters" => {
                            builder = builder.set_kinesis_parameters(crate::protocol_serde::shape_kinesis_parameters::de_kinesis_parameters(tokens)?);
                        }
                        "RunCommandParameters" => {
                            builder = builder
                                .set_run_command_parameters(crate::protocol_serde::shape_run_command_parameters::de_run_command_parameters(tokens)?);
                        }
                        "EcsParameters" => {
                            builder = builder.set_ecs_parameters(crate::protocol_serde::shape_ecs_parameters::de_ecs_parameters(tokens)?);
                        }
                        "BatchParameters" => {
                            builder = builder.set_batch_parameters(crate::protocol_serde::shape_batch_parameters::de_batch_parameters(tokens)?);
                        }
                        "SqsParameters" => {
                            builder = builder.set_sqs_parameters(crate::protocol_serde::shape_sqs_parameters::de_sqs_parameters(tokens)?);
                        }
                        "HttpParameters" => {
                            builder = builder.set_http_parameters(crate::protocol_serde::shape_http_parameters::de_http_parameters(tokens)?);
                        }
                        "RedshiftDataParameters" => {
                            builder = builder.set_redshift_data_parameters(
                                crate::protocol_serde::shape_redshift_data_parameters::de_redshift_data_parameters(tokens)?,
                            );
                        }
                        "SageMakerPipelineParameters" => {
                            builder = builder.set_sage_maker_pipeline_parameters(
                                crate::protocol_serde::shape_sage_maker_pipeline_parameters::de_sage_maker_pipeline_parameters(tokens)?,
                            );
                        }
                        "DeadLetterConfig" => {
                            builder = builder.set_dead_letter_config(crate::protocol_serde::shape_dead_letter_config::de_dead_letter_config(tokens)?);
                        }
                        "RetryPolicy" => {
                            builder = builder.set_retry_policy(crate::protocol_serde::shape_retry_policy::de_retry_policy(tokens)?);
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
            Ok(Some(crate::serde_util::target_correct_errors(builder).build().map_err(|err| {
                ::aws_smithy_json::deserialize::error::DeserializeError::custom_source("Response was invalid", err)
            })?))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

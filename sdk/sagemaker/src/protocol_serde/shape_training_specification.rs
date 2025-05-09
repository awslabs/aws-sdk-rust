// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_training_specification(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::TrainingSpecification,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.training_image {
        object.key("TrainingImage").string(var_1.as_str());
    }
    if let Some(var_2) = &input.training_image_digest {
        object.key("TrainingImageDigest").string(var_2.as_str());
    }
    if let Some(var_3) = &input.supported_hyper_parameters {
        let mut array_4 = object.key("SupportedHyperParameters").start_array();
        for item_5 in var_3 {
            {
                #[allow(unused_mut)]
                let mut object_6 = array_4.value().start_object();
                crate::protocol_serde::shape_hyper_parameter_specification::ser_hyper_parameter_specification(&mut object_6, item_5)?;
                object_6.finish();
            }
        }
        array_4.finish();
    }
    if let Some(var_7) = &input.supported_training_instance_types {
        let mut array_8 = object.key("SupportedTrainingInstanceTypes").start_array();
        for item_9 in var_7 {
            {
                array_8.value().string(item_9.as_str());
            }
        }
        array_8.finish();
    }
    if let Some(var_10) = &input.supports_distributed_training {
        object.key("SupportsDistributedTraining").boolean(*var_10);
    }
    if let Some(var_11) = &input.metric_definitions {
        let mut array_12 = object.key("MetricDefinitions").start_array();
        for item_13 in var_11 {
            {
                #[allow(unused_mut)]
                let mut object_14 = array_12.value().start_object();
                crate::protocol_serde::shape_metric_definition::ser_metric_definition(&mut object_14, item_13)?;
                object_14.finish();
            }
        }
        array_12.finish();
    }
    if let Some(var_15) = &input.training_channels {
        let mut array_16 = object.key("TrainingChannels").start_array();
        for item_17 in var_15 {
            {
                #[allow(unused_mut)]
                let mut object_18 = array_16.value().start_object();
                crate::protocol_serde::shape_channel_specification::ser_channel_specification(&mut object_18, item_17)?;
                object_18.finish();
            }
        }
        array_16.finish();
    }
    if let Some(var_19) = &input.supported_tuning_job_objective_metrics {
        let mut array_20 = object.key("SupportedTuningJobObjectiveMetrics").start_array();
        for item_21 in var_19 {
            {
                #[allow(unused_mut)]
                let mut object_22 = array_20.value().start_object();
                crate::protocol_serde::shape_hyper_parameter_tuning_job_objective::ser_hyper_parameter_tuning_job_objective(&mut object_22, item_21)?;
                object_22.finish();
            }
        }
        array_20.finish();
    }
    if let Some(var_23) = &input.additional_s3_data_source {
        #[allow(unused_mut)]
        let mut object_24 = object.key("AdditionalS3DataSource").start_object();
        crate::protocol_serde::shape_additional_s3_data_source::ser_additional_s3_data_source(&mut object_24, var_23)?;
        object_24.finish();
    }
    Ok(())
}

pub(crate) fn de_training_specification<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::TrainingSpecification>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::TrainingSpecificationBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "TrainingImage" => {
                            builder = builder.set_training_image(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "TrainingImageDigest" => {
                            builder = builder.set_training_image_digest(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "SupportedHyperParameters" => {
                            builder = builder.set_supported_hyper_parameters(
                                crate::protocol_serde::shape_hyper_parameter_specifications::de_hyper_parameter_specifications(tokens)?,
                            );
                        }
                        "SupportedTrainingInstanceTypes" => {
                            builder = builder.set_supported_training_instance_types(
                                crate::protocol_serde::shape_training_instance_types::de_training_instance_types(tokens)?,
                            );
                        }
                        "SupportsDistributedTraining" => {
                            builder =
                                builder.set_supports_distributed_training(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "MetricDefinitions" => {
                            builder = builder
                                .set_metric_definitions(crate::protocol_serde::shape_metric_definition_list::de_metric_definition_list(tokens)?);
                        }
                        "TrainingChannels" => {
                            builder = builder
                                .set_training_channels(crate::protocol_serde::shape_channel_specifications::de_channel_specifications(tokens)?);
                        }
                        "SupportedTuningJobObjectiveMetrics" => {
                            builder = builder.set_supported_tuning_job_objective_metrics(
                                crate::protocol_serde::shape_hyper_parameter_tuning_job_objectives::de_hyper_parameter_tuning_job_objectives(tokens)?,
                            );
                        }
                        "AdditionalS3DataSource" => {
                            builder = builder.set_additional_s3_data_source(
                                crate::protocol_serde::shape_additional_s3_data_source::de_additional_s3_data_source(tokens)?,
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
            Ok(Some(crate::serde_util::training_specification_correct_errors(builder).build()))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

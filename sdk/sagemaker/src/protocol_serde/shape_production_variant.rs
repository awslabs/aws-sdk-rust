// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_production_variant(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ProductionVariant,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.variant_name {
        object.key("VariantName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.model_name {
        object.key("ModelName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.initial_instance_count {
        object.key("InitialInstanceCount").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_3).into()),
        );
    }
    if let Some(var_4) = &input.instance_type {
        object.key("InstanceType").string(var_4.as_str());
    }
    if let Some(var_5) = &input.initial_variant_weight {
        object.key("InitialVariantWeight").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::Float((*var_5).into()),
        );
    }
    if let Some(var_6) = &input.accelerator_type {
        object.key("AcceleratorType").string(var_6.as_str());
    }
    if let Some(var_7) = &input.core_dump_config {
        #[allow(unused_mut)]
        let mut object_8 = object.key("CoreDumpConfig").start_object();
        crate::protocol_serde::shape_production_variant_core_dump_config::ser_production_variant_core_dump_config(&mut object_8, var_7)?;
        object_8.finish();
    }
    if let Some(var_9) = &input.serverless_config {
        #[allow(unused_mut)]
        let mut object_10 = object.key("ServerlessConfig").start_object();
        crate::protocol_serde::shape_production_variant_serverless_config::ser_production_variant_serverless_config(&mut object_10, var_9)?;
        object_10.finish();
    }
    if let Some(var_11) = &input.volume_size_in_gb {
        object.key("VolumeSizeInGB").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_11).into()),
        );
    }
    if let Some(var_12) = &input.model_data_download_timeout_in_seconds {
        object.key("ModelDataDownloadTimeoutInSeconds").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_12).into()),
        );
    }
    if let Some(var_13) = &input.container_startup_health_check_timeout_in_seconds {
        object.key("ContainerStartupHealthCheckTimeoutInSeconds").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_13).into()),
        );
    }
    if let Some(var_14) = &input.enable_ssm_access {
        object.key("EnableSSMAccess").boolean(*var_14);
    }
    if let Some(var_15) = &input.managed_instance_scaling {
        #[allow(unused_mut)]
        let mut object_16 = object.key("ManagedInstanceScaling").start_object();
        crate::protocol_serde::shape_production_variant_managed_instance_scaling::ser_production_variant_managed_instance_scaling(
            &mut object_16,
            var_15,
        )?;
        object_16.finish();
    }
    if let Some(var_17) = &input.routing_config {
        #[allow(unused_mut)]
        let mut object_18 = object.key("RoutingConfig").start_object();
        crate::protocol_serde::shape_production_variant_routing_config::ser_production_variant_routing_config(&mut object_18, var_17)?;
        object_18.finish();
    }
    if let Some(var_19) = &input.inference_ami_version {
        object.key("InferenceAmiVersion").string(var_19.as_str());
    }
    if let Some(var_20) = &input.capacity_reservation_config {
        #[allow(unused_mut)]
        let mut object_21 = object.key("CapacityReservationConfig").start_object();
        crate::protocol_serde::shape_production_variant_capacity_reservation_config::ser_production_variant_capacity_reservation_config(
            &mut object_21,
            var_20,
        )?;
        object_21.finish();
    }
    Ok(())
}

pub(crate) fn de_production_variant<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::ProductionVariant>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::ProductionVariantBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "VariantName" => {
                            builder = builder.set_variant_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "ModelName" => {
                            builder = builder.set_model_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "InitialInstanceCount" => {
                            builder = builder.set_initial_instance_count(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "InstanceType" => {
                            builder = builder.set_instance_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::ProductionVariantInstanceType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "InitialVariantWeight" => {
                            builder = builder.set_initial_variant_weight(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?.map(|v| v.to_f32_lossy()),
                            );
                        }
                        "AcceleratorType" => {
                            builder = builder.set_accelerator_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::ProductionVariantAcceleratorType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "CoreDumpConfig" => {
                            builder = builder.set_core_dump_config(
                                crate::protocol_serde::shape_production_variant_core_dump_config::de_production_variant_core_dump_config(tokens)?,
                            );
                        }
                        "ServerlessConfig" => {
                            builder = builder.set_serverless_config(
                                crate::protocol_serde::shape_production_variant_serverless_config::de_production_variant_serverless_config(tokens)?,
                            );
                        }
                        "VolumeSizeInGB" => {
                            builder = builder.set_volume_size_in_gb(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "ModelDataDownloadTimeoutInSeconds" => {
                            builder = builder.set_model_data_download_timeout_in_seconds(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "ContainerStartupHealthCheckTimeoutInSeconds" => {
                            builder = builder.set_container_startup_health_check_timeout_in_seconds(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "EnableSSMAccess" => {
                            builder = builder.set_enable_ssm_access(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "ManagedInstanceScaling" => {
                            builder = builder.set_managed_instance_scaling(
                                    crate::protocol_serde::shape_production_variant_managed_instance_scaling::de_production_variant_managed_instance_scaling(tokens)?
                                );
                        }
                        "RoutingConfig" => {
                            builder = builder.set_routing_config(
                                crate::protocol_serde::shape_production_variant_routing_config::de_production_variant_routing_config(tokens)?,
                            );
                        }
                        "InferenceAmiVersion" => {
                            builder = builder.set_inference_ami_version(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| {
                                        s.to_unescaped()
                                            .map(|u| crate::types::ProductionVariantInferenceAmiVersion::from(u.as_ref()))
                                    })
                                    .transpose()?,
                            );
                        }
                        "CapacityReservationConfig" => {
                            builder = builder.set_capacity_reservation_config(
                                    crate::protocol_serde::shape_production_variant_capacity_reservation_config::de_production_variant_capacity_reservation_config(tokens)?
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
            Ok(Some(crate::serde_util::production_variant_correct_errors(builder).build()))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

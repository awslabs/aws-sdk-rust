// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_inference_component_compute_resource_requirements(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::InferenceComponentComputeResourceRequirements,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.number_of_cpu_cores_required {
        object.key("NumberOfCpuCoresRequired").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::Float((*var_1).into()),
        );
    }
    if let Some(var_2) = &input.number_of_accelerator_devices_required {
        object.key("NumberOfAcceleratorDevicesRequired").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::Float((*var_2).into()),
        );
    }
    if let Some(var_3) = &input.min_memory_required_in_mb {
        object.key("MinMemoryRequiredInMb").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_3).into()),
        );
    }
    if let Some(var_4) = &input.max_memory_required_in_mb {
        object.key("MaxMemoryRequiredInMb").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_4).into()),
        );
    }
    Ok(())
}

pub(crate) fn de_inference_component_compute_resource_requirements<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::InferenceComponentComputeResourceRequirements>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::InferenceComponentComputeResourceRequirementsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "NumberOfCpuCoresRequired" => {
                            builder = builder.set_number_of_cpu_cores_required(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?.map(|v| v.to_f32_lossy()),
                            );
                        }
                        "NumberOfAcceleratorDevicesRequired" => {
                            builder = builder.set_number_of_accelerator_devices_required(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?.map(|v| v.to_f32_lossy()),
                            );
                        }
                        "MinMemoryRequiredInMb" => {
                            builder = builder.set_min_memory_required_in_mb(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "MaxMemoryRequiredInMb" => {
                            builder = builder.set_max_memory_required_in_mb(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
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
                crate::serde_util::inference_component_compute_resource_requirements_correct_errors(builder).build(),
            ))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

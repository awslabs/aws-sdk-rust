// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_payment_configuration(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::PaymentConfiguration,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.query_compute {
        #[allow(unused_mut)]
        let mut object_2 = object.key("queryCompute").start_object();
        crate::protocol_serde::shape_query_compute_payment_config::ser_query_compute_payment_config(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.machine_learning {
        #[allow(unused_mut)]
        let mut object_4 = object.key("machineLearning").start_object();
        crate::protocol_serde::shape_ml_payment_config::ser_ml_payment_config(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.job_compute {
        #[allow(unused_mut)]
        let mut object_6 = object.key("jobCompute").start_object();
        crate::protocol_serde::shape_job_compute_payment_config::ser_job_compute_payment_config(&mut object_6, var_5)?;
        object_6.finish();
    }
    Ok(())
}

pub(crate) fn de_payment_configuration<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::PaymentConfiguration>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::PaymentConfigurationBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "queryCompute" => {
                            builder = builder.set_query_compute(
                                crate::protocol_serde::shape_query_compute_payment_config::de_query_compute_payment_config(tokens)?,
                            );
                        }
                        "machineLearning" => {
                            builder = builder.set_machine_learning(crate::protocol_serde::shape_ml_payment_config::de_ml_payment_config(tokens)?);
                        }
                        "jobCompute" => {
                            builder = builder.set_job_compute(
                                crate::protocol_serde::shape_job_compute_payment_config::de_job_compute_payment_config(tokens)?,
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
            Ok(Some(crate::serde_util::payment_configuration_correct_errors(builder).build()))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

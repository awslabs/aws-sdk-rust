// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_aws_ecs_cluster_configuration_details(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::AwsEcsClusterConfigurationDetails,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.execute_command_configuration {
        #[allow(unused_mut)]
        let mut object_2 = object.key("ExecuteCommandConfiguration").start_object();
        crate::protocol_serde::shape_aws_ecs_cluster_configuration_execute_command_configuration_details::ser_aws_ecs_cluster_configuration_execute_command_configuration_details(&mut object_2, var_1)?;
        object_2.finish();
    }
    Ok(())
}

pub(crate) fn de_aws_ecs_cluster_configuration_details<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::AwsEcsClusterConfigurationDetails>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::AwsEcsClusterConfigurationDetailsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "ExecuteCommandConfiguration" => {
                            builder = builder.set_execute_command_configuration(
                                    crate::protocol_serde::shape_aws_ecs_cluster_configuration_execute_command_configuration_details::de_aws_ecs_cluster_configuration_execute_command_configuration_details(tokens)?
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

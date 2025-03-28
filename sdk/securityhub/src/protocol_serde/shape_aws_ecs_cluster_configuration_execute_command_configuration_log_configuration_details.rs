// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_aws_ecs_cluster_configuration_execute_command_configuration_log_configuration_details(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::AwsEcsClusterConfigurationExecuteCommandConfigurationLogConfigurationDetails,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.cloud_watch_encryption_enabled {
        object.key("CloudWatchEncryptionEnabled").boolean(*var_1);
    }
    if let Some(var_2) = &input.cloud_watch_log_group_name {
        object.key("CloudWatchLogGroupName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.s3_bucket_name {
        object.key("S3BucketName").string(var_3.as_str());
    }
    if let Some(var_4) = &input.s3_encryption_enabled {
        object.key("S3EncryptionEnabled").boolean(*var_4);
    }
    if let Some(var_5) = &input.s3_key_prefix {
        object.key("S3KeyPrefix").string(var_5.as_str());
    }
    Ok(())
}

pub(crate) fn de_aws_ecs_cluster_configuration_execute_command_configuration_log_configuration_details<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<
    Option<crate::types::AwsEcsClusterConfigurationExecuteCommandConfigurationLogConfigurationDetails>,
    ::aws_smithy_json::deserialize::error::DeserializeError,
>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::AwsEcsClusterConfigurationExecuteCommandConfigurationLogConfigurationDetailsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "CloudWatchEncryptionEnabled" => {
                            builder = builder
                                .set_cloud_watch_encryption_enabled(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "CloudWatchLogGroupName" => {
                            builder = builder.set_cloud_watch_log_group_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "S3BucketName" => {
                            builder = builder.set_s3_bucket_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "S3EncryptionEnabled" => {
                            builder = builder.set_s3_encryption_enabled(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "S3KeyPrefix" => {
                            builder = builder.set_s3_key_prefix(
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

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_recovery_instance<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::RecoveryInstance>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::RecoveryInstanceBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "ec2InstanceID" => {
                            builder = builder.set_ec2_instance_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "ec2InstanceState" => {
                            builder = builder.set_ec2_instance_state(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::Ec2InstanceState::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "jobID" => {
                            builder = builder.set_job_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "recoveryInstanceID" => {
                            builder = builder.set_recovery_instance_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "sourceServerID" => {
                            builder = builder.set_source_server_id(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "arn" => {
                            builder = builder.set_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "tags" => {
                            builder = builder.set_tags(crate::protocol_serde::shape_tags_map::de_tags_map(tokens)?);
                        }
                        "failback" => {
                            builder = builder.set_failback(crate::protocol_serde::shape_recovery_instance_failback::de_recovery_instance_failback(
                                tokens,
                            )?);
                        }
                        "dataReplicationInfo" => {
                            builder = builder.set_data_replication_info(
                                crate::protocol_serde::shape_recovery_instance_data_replication_info::de_recovery_instance_data_replication_info(
                                    tokens,
                                )?,
                            );
                        }
                        "recoveryInstanceProperties" => {
                            builder = builder.set_recovery_instance_properties(
                                crate::protocol_serde::shape_recovery_instance_properties::de_recovery_instance_properties(tokens)?,
                            );
                        }
                        "pointInTimeSnapshotDateTime" => {
                            builder = builder.set_point_in_time_snapshot_date_time(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "isDrill" => {
                            builder = builder.set_is_drill(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "originEnvironment" => {
                            builder = builder.set_origin_environment(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::OriginEnvironment::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "originAvailabilityZone" => {
                            builder = builder.set_origin_availability_zone(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "agentVersion" => {
                            builder = builder.set_agent_version(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "sourceOutpostArn" => {
                            builder = builder.set_source_outpost_arn(
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

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_configuration_recorder<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::ConfigurationRecorder>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::ConfigurationRecorderBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "arn" => {
                            builder = builder.set_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "name" => {
                            builder = builder.set_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "roleARN" => {
                            builder = builder.set_role_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "recordingGroup" => {
                            builder = builder.set_recording_group(crate::protocol_serde::shape_recording_group::de_recording_group(tokens)?);
                        }
                        "recordingMode" => {
                            builder = builder.set_recording_mode(crate::protocol_serde::shape_recording_mode::de_recording_mode(tokens)?);
                        }
                        "recordingScope" => {
                            builder = builder.set_recording_scope(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::RecordingScope::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "servicePrincipal" => {
                            builder = builder.set_service_principal(
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

pub fn ser_configuration_recorder(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ConfigurationRecorder,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.arn {
        object.key("arn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.name {
        object.key("name").string(var_2.as_str());
    }
    if let Some(var_3) = &input.role_arn {
        object.key("roleARN").string(var_3.as_str());
    }
    if let Some(var_4) = &input.recording_group {
        #[allow(unused_mut)]
        let mut object_5 = object.key("recordingGroup").start_object();
        crate::protocol_serde::shape_recording_group::ser_recording_group(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.recording_mode {
        #[allow(unused_mut)]
        let mut object_7 = object.key("recordingMode").start_object();
        crate::protocol_serde::shape_recording_mode::ser_recording_mode(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.recording_scope {
        object.key("recordingScope").string(var_8.as_str());
    }
    if let Some(var_9) = &input.service_principal {
        object.key("servicePrincipal").string(var_9.as_str());
    }
    Ok(())
}

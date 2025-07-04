// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_encryption<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::Encryption>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::EncryptionBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "ConstantInitializationVector" => {
                            builder = builder.set_constant_initialization_vector(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "EncryptionMethod" => {
                            builder = builder.set_encryption_method(crate::protocol_serde::shape_encryption_method::de_encryption_method(tokens)?);
                        }
                        "KeyRotationIntervalSeconds" => {
                            builder = builder.set_key_rotation_interval_seconds(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "CmafExcludeSegmentDrmMetadata" => {
                            builder = builder
                                .set_cmaf_exclude_segment_drm_metadata(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "SpekeKeyProvider" => {
                            builder = builder.set_speke_key_provider(crate::protocol_serde::shape_speke_key_provider::de_speke_key_provider(tokens)?);
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
            Ok(Some(crate::serde_util::encryption_correct_errors(builder).build()))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

pub fn ser_encryption(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::Encryption,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.constant_initialization_vector {
        object.key("ConstantInitializationVector").string(var_1.as_str());
    }
    if let Some(var_2) = &input.encryption_method {
        #[allow(unused_mut)]
        let mut object_3 = object.key("EncryptionMethod").start_object();
        crate::protocol_serde::shape_encryption_method::ser_encryption_method(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.key_rotation_interval_seconds {
        object.key("KeyRotationIntervalSeconds").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_4).into()),
        );
    }
    if let Some(var_5) = &input.cmaf_exclude_segment_drm_metadata {
        object.key("CmafExcludeSegmentDrmMetadata").boolean(*var_5);
    }
    if let Some(var_6) = &input.speke_key_provider {
        #[allow(unused_mut)]
        let mut object_7 = object.key("SpekeKeyProvider").start_object();
        crate::protocol_serde::shape_speke_key_provider::ser_speke_key_provider(&mut object_7, var_6)?;
        object_7.finish();
    }
    Ok(())
}

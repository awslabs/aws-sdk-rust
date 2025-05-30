// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_s3_output_format_config(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::S3OutputFormatConfig,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.file_type {
        object.key("fileType").string(var_1.as_str());
    }
    if let Some(var_2) = &input.prefix_config {
        #[allow(unused_mut)]
        let mut object_3 = object.key("prefixConfig").start_object();
        crate::protocol_serde::shape_prefix_config::ser_prefix_config(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.aggregation_config {
        #[allow(unused_mut)]
        let mut object_5 = object.key("aggregationConfig").start_object();
        crate::protocol_serde::shape_aggregation_config::ser_aggregation_config(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.preserve_source_data_typing {
        object.key("preserveSourceDataTyping").boolean(*var_6);
    }
    Ok(())
}

pub(crate) fn de_s3_output_format_config<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::S3OutputFormatConfig>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::S3OutputFormatConfigBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "fileType" => {
                            builder = builder.set_file_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::FileType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "prefixConfig" => {
                            builder = builder.set_prefix_config(crate::protocol_serde::shape_prefix_config::de_prefix_config(tokens)?);
                        }
                        "aggregationConfig" => {
                            builder = builder.set_aggregation_config(crate::protocol_serde::shape_aggregation_config::de_aggregation_config(tokens)?);
                        }
                        "preserveSourceDataTyping" => {
                            builder =
                                builder.set_preserve_source_data_typing(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
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

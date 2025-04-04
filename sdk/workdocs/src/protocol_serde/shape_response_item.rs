// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_response_item<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::ResponseItem>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::ResponseItemBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "ResourceType" => {
                            builder = builder.set_resource_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| crate::types::ResponseItemType::from(u.as_ref())))
                                    .transpose()?,
                            );
                        }
                        "WebUrl" => {
                            builder = builder.set_web_url(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "DocumentMetadata" => {
                            builder = builder.set_document_metadata(crate::protocol_serde::shape_document_metadata::de_document_metadata(tokens)?);
                        }
                        "FolderMetadata" => {
                            builder = builder.set_folder_metadata(crate::protocol_serde::shape_folder_metadata::de_folder_metadata(tokens)?);
                        }
                        "CommentMetadata" => {
                            builder = builder.set_comment_metadata(crate::protocol_serde::shape_comment_metadata::de_comment_metadata(tokens)?);
                        }
                        "DocumentVersionMetadata" => {
                            builder = builder.set_document_version_metadata(
                                crate::protocol_serde::shape_document_version_metadata::de_document_version_metadata(tokens)?,
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

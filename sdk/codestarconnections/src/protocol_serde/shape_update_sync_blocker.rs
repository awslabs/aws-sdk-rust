// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_update_sync_blocker_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::update_sync_blocker::UpdateSyncBlockerOutput, crate::operation::update_sync_blocker::UpdateSyncBlockerError>
{
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::update_sync_blocker::UpdateSyncBlockerError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::update_sync_blocker::UpdateSyncBlockerError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AccessDeniedException" => crate::operation::update_sync_blocker::UpdateSyncBlockerError::AccessDeniedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AccessDeniedExceptionBuilder::default();
                output = crate::protocol_serde::shape_access_denied_exception::de_access_denied_exception_json_err(_response_body, output)
                    .map_err(crate::operation::update_sync_blocker::UpdateSyncBlockerError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalServerException" => crate::operation::update_sync_blocker::UpdateSyncBlockerError::InternalServerException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_server_exception::de_internal_server_exception_json_err(_response_body, output)
                    .map_err(crate::operation::update_sync_blocker::UpdateSyncBlockerError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidInputException" => crate::operation::update_sync_blocker::UpdateSyncBlockerError::InvalidInputException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidInputExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_input_exception::de_invalid_input_exception_json_err(_response_body, output)
                    .map_err(crate::operation::update_sync_blocker::UpdateSyncBlockerError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceNotFoundException" => crate::operation::update_sync_blocker::UpdateSyncBlockerError::ResourceNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::update_sync_blocker::UpdateSyncBlockerError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "RetryLatestCommitFailedException" => crate::operation::update_sync_blocker::UpdateSyncBlockerError::RetryLatestCommitFailedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::RetryLatestCommitFailedExceptionBuilder::default();
                output = crate::protocol_serde::shape_retry_latest_commit_failed_exception::de_retry_latest_commit_failed_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::update_sync_blocker::UpdateSyncBlockerError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "SyncBlockerDoesNotExistException" => crate::operation::update_sync_blocker::UpdateSyncBlockerError::SyncBlockerDoesNotExistException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::SyncBlockerDoesNotExistExceptionBuilder::default();
                output = crate::protocol_serde::shape_sync_blocker_does_not_exist_exception::de_sync_blocker_does_not_exist_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::update_sync_blocker::UpdateSyncBlockerError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ThrottlingException" => crate::operation::update_sync_blocker::UpdateSyncBlockerError::ThrottlingException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingExceptionBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(_response_body, output)
                    .map_err(crate::operation::update_sync_blocker::UpdateSyncBlockerError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::update_sync_blocker::UpdateSyncBlockerError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_update_sync_blocker_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::update_sync_blocker::UpdateSyncBlockerOutput, crate::operation::update_sync_blocker::UpdateSyncBlockerError>
{
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::update_sync_blocker::builders::UpdateSyncBlockerOutputBuilder::default();
        output = crate::protocol_serde::shape_update_sync_blocker::de_update_sync_blocker(_response_body, output)
            .map_err(crate::operation::update_sync_blocker::UpdateSyncBlockerError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::update_sync_blocker_output_output_correct_errors(output)
            .build()
            .map_err(crate::operation::update_sync_blocker::UpdateSyncBlockerError::unhandled)?
    })
}

pub fn ser_update_sync_blocker_input(
    input: &crate::operation::update_sync_blocker::UpdateSyncBlockerInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_update_sync_blocker_input::ser_update_sync_blocker_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_update_sync_blocker(
    value: &[u8],
    mut builder: crate::operation::update_sync_blocker::builders::UpdateSyncBlockerOutputBuilder,
) -> ::std::result::Result<
    crate::operation::update_sync_blocker::builders::UpdateSyncBlockerOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "ResourceName" => {
                    builder = builder.set_resource_name(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "ParentResourceName" => {
                    builder = builder.set_parent_resource_name(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "SyncBlocker" => {
                    builder = builder.set_sync_blocker(crate::protocol_serde::shape_sync_blocker::de_sync_blocker(tokens)?);
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
    if tokens.next().is_some() {
        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "found more JSON tokens after completing parsing",
        ));
    }
    Ok(builder)
}

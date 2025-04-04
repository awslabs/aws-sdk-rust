// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_restore_table_from_backup_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::restore_table_from_backup::RestoreTableFromBackupOutput,
    crate::operation::restore_table_from_backup::RestoreTableFromBackupError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::restore_table_from_backup::RestoreTableFromBackupError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => {
            return Err(crate::operation::restore_table_from_backup::RestoreTableFromBackupError::unhandled(
                generic,
            ))
        }
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "BackupInUseException" => crate::operation::restore_table_from_backup::RestoreTableFromBackupError::BackupInUseException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::BackupInUseExceptionBuilder::default();
                output = crate::protocol_serde::shape_backup_in_use_exception::de_backup_in_use_exception_json_err(_response_body, output)
                    .map_err(crate::operation::restore_table_from_backup::RestoreTableFromBackupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "BackupNotFoundException" => crate::operation::restore_table_from_backup::RestoreTableFromBackupError::BackupNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::BackupNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_backup_not_found_exception::de_backup_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::restore_table_from_backup::RestoreTableFromBackupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalServerError" => crate::operation::restore_table_from_backup::RestoreTableFromBackupError::InternalServerError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerErrorBuilder::default();
                output = crate::protocol_serde::shape_internal_server_error::de_internal_server_error_json_err(_response_body, output)
                    .map_err(crate::operation::restore_table_from_backup::RestoreTableFromBackupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidEndpointException" => crate::operation::restore_table_from_backup::RestoreTableFromBackupError::InvalidEndpointException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidEndpointExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_endpoint_exception::de_invalid_endpoint_exception_json_err(_response_body, output)
                    .map_err(crate::operation::restore_table_from_backup::RestoreTableFromBackupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "LimitExceededException" => crate::operation::restore_table_from_backup::RestoreTableFromBackupError::LimitExceededException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::LimitExceededExceptionBuilder::default();
                output = crate::protocol_serde::shape_limit_exceeded_exception::de_limit_exceeded_exception_json_err(_response_body, output)
                    .map_err(crate::operation::restore_table_from_backup::RestoreTableFromBackupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TableAlreadyExistsException" => crate::operation::restore_table_from_backup::RestoreTableFromBackupError::TableAlreadyExistsException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::TableAlreadyExistsExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_table_already_exists_exception::de_table_already_exists_exception_json_err(_response_body, output)
                        .map_err(crate::operation::restore_table_from_backup::RestoreTableFromBackupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TableInUseException" => crate::operation::restore_table_from_backup::RestoreTableFromBackupError::TableInUseException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::TableInUseExceptionBuilder::default();
                output = crate::protocol_serde::shape_table_in_use_exception::de_table_in_use_exception_json_err(_response_body, output)
                    .map_err(crate::operation::restore_table_from_backup::RestoreTableFromBackupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::restore_table_from_backup::RestoreTableFromBackupError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_restore_table_from_backup_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::restore_table_from_backup::RestoreTableFromBackupOutput,
    crate::operation::restore_table_from_backup::RestoreTableFromBackupError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::restore_table_from_backup::builders::RestoreTableFromBackupOutputBuilder::default();
        output = crate::protocol_serde::shape_restore_table_from_backup::de_restore_table_from_backup(_response_body, output)
            .map_err(crate::operation::restore_table_from_backup::RestoreTableFromBackupError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_restore_table_from_backup_input(
    input: &crate::operation::restore_table_from_backup::RestoreTableFromBackupInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_restore_table_from_backup_input::ser_restore_table_from_backup_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_restore_table_from_backup(
    value: &[u8],
    mut builder: crate::operation::restore_table_from_backup::builders::RestoreTableFromBackupOutputBuilder,
) -> ::std::result::Result<
    crate::operation::restore_table_from_backup::builders::RestoreTableFromBackupOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "TableDescription" => {
                    builder = builder.set_table_description(crate::protocol_serde::shape_table_description::de_table_description(tokens)?);
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

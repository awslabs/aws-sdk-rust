// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_backup_policy_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::describe_backup_policy::DescribeBackupPolicyOutput,
    crate::operation::describe_backup_policy::DescribeBackupPolicyError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::describe_backup_policy::DescribeBackupPolicyError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::describe_backup_policy::DescribeBackupPolicyError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "BadRequest" => crate::operation::describe_backup_policy::DescribeBackupPolicyError::BadRequest({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::BadRequestBuilder::default();
                output = crate::protocol_serde::shape_bad_request::de_bad_request_json_err(_response_body, output)
                    .map_err(crate::operation::describe_backup_policy::DescribeBackupPolicyError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::bad_request_correct_errors(output)
                    .build()
                    .map_err(crate::operation::describe_backup_policy::DescribeBackupPolicyError::unhandled)?
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "FileSystemNotFound" => crate::operation::describe_backup_policy::DescribeBackupPolicyError::FileSystemNotFound({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::FileSystemNotFoundBuilder::default();
                output = crate::protocol_serde::shape_file_system_not_found::de_file_system_not_found_json_err(_response_body, output)
                    .map_err(crate::operation::describe_backup_policy::DescribeBackupPolicyError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::file_system_not_found_correct_errors(output)
                    .build()
                    .map_err(crate::operation::describe_backup_policy::DescribeBackupPolicyError::unhandled)?
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalServerError" => crate::operation::describe_backup_policy::DescribeBackupPolicyError::InternalServerError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerErrorBuilder::default();
                output = crate::protocol_serde::shape_internal_server_error::de_internal_server_error_json_err(_response_body, output)
                    .map_err(crate::operation::describe_backup_policy::DescribeBackupPolicyError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::internal_server_error_correct_errors(output)
                    .build()
                    .map_err(crate::operation::describe_backup_policy::DescribeBackupPolicyError::unhandled)?
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "PolicyNotFound" => crate::operation::describe_backup_policy::DescribeBackupPolicyError::PolicyNotFound({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::PolicyNotFoundBuilder::default();
                output = crate::protocol_serde::shape_policy_not_found::de_policy_not_found_json_err(_response_body, output)
                    .map_err(crate::operation::describe_backup_policy::DescribeBackupPolicyError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ValidationException" => crate::operation::describe_backup_policy::DescribeBackupPolicyError::ValidationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationExceptionBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_backup_policy::DescribeBackupPolicyError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::validation_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::describe_backup_policy::DescribeBackupPolicyError::unhandled)?
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::describe_backup_policy::DescribeBackupPolicyError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_backup_policy_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::describe_backup_policy::DescribeBackupPolicyOutput,
    crate::operation::describe_backup_policy::DescribeBackupPolicyError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::describe_backup_policy::builders::DescribeBackupPolicyOutputBuilder::default();
        output = crate::protocol_serde::shape_describe_backup_policy::de_describe_backup_policy(_response_body, output)
            .map_err(crate::operation::describe_backup_policy::DescribeBackupPolicyError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub(crate) fn de_describe_backup_policy(
    value: &[u8],
    mut builder: crate::operation::describe_backup_policy::builders::DescribeBackupPolicyOutputBuilder,
) -> ::std::result::Result<
    crate::operation::describe_backup_policy::builders::DescribeBackupPolicyOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "BackupPolicy" => {
                    builder = builder.set_backup_policy(crate::protocol_serde::shape_backup_policy::de_backup_policy(tokens)?);
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

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_list_pull_requests_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::list_pull_requests::ListPullRequestsOutput, crate::operation::list_pull_requests::ListPullRequestsError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AuthorDoesNotExistException" => crate::operation::list_pull_requests::ListPullRequestsError::AuthorDoesNotExistException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AuthorDoesNotExistExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_author_does_not_exist_exception::de_author_does_not_exist_exception_json_err(_response_body, output)
                        .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "EncryptionIntegrityChecksFailedException" => {
            crate::operation::list_pull_requests::ListPullRequestsError::EncryptionIntegrityChecksFailedException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::EncryptionIntegrityChecksFailedExceptionBuilder::default();
                    output = crate::protocol_serde::shape_encryption_integrity_checks_failed_exception::de_encryption_integrity_checks_failed_exception_json_err(_response_body, output).map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "EncryptionKeyAccessDeniedException" => crate::operation::list_pull_requests::ListPullRequestsError::EncryptionKeyAccessDeniedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::EncryptionKeyAccessDeniedExceptionBuilder::default();
                output = crate::protocol_serde::shape_encryption_key_access_denied_exception::de_encryption_key_access_denied_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "EncryptionKeyDisabledException" => crate::operation::list_pull_requests::ListPullRequestsError::EncryptionKeyDisabledException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::EncryptionKeyDisabledExceptionBuilder::default();
                output = crate::protocol_serde::shape_encryption_key_disabled_exception::de_encryption_key_disabled_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "EncryptionKeyNotFoundException" => crate::operation::list_pull_requests::ListPullRequestsError::EncryptionKeyNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::EncryptionKeyNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_encryption_key_not_found_exception::de_encryption_key_not_found_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "EncryptionKeyUnavailableException" => crate::operation::list_pull_requests::ListPullRequestsError::EncryptionKeyUnavailableException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::EncryptionKeyUnavailableExceptionBuilder::default();
                output = crate::protocol_serde::shape_encryption_key_unavailable_exception::de_encryption_key_unavailable_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidAuthorArnException" => crate::operation::list_pull_requests::ListPullRequestsError::InvalidAuthorArnException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidAuthorArnExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_author_arn_exception::de_invalid_author_arn_exception_json_err(_response_body, output)
                    .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidContinuationTokenException" => crate::operation::list_pull_requests::ListPullRequestsError::InvalidContinuationTokenException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidContinuationTokenExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_continuation_token_exception::de_invalid_continuation_token_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidMaxResultsException" => crate::operation::list_pull_requests::ListPullRequestsError::InvalidMaxResultsException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidMaxResultsExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_invalid_max_results_exception::de_invalid_max_results_exception_json_err(_response_body, output)
                        .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidPullRequestStatusException" => crate::operation::list_pull_requests::ListPullRequestsError::InvalidPullRequestStatusException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidPullRequestStatusExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_pull_request_status_exception::de_invalid_pull_request_status_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidRepositoryNameException" => crate::operation::list_pull_requests::ListPullRequestsError::InvalidRepositoryNameException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidRepositoryNameExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_repository_name_exception::de_invalid_repository_name_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "RepositoryDoesNotExistException" => crate::operation::list_pull_requests::ListPullRequestsError::RepositoryDoesNotExistException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::RepositoryDoesNotExistExceptionBuilder::default();
                output = crate::protocol_serde::shape_repository_does_not_exist_exception::de_repository_does_not_exist_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "RepositoryNameRequiredException" => crate::operation::list_pull_requests::ListPullRequestsError::RepositoryNameRequiredException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::RepositoryNameRequiredExceptionBuilder::default();
                output = crate::protocol_serde::shape_repository_name_required_exception::de_repository_name_required_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::list_pull_requests::ListPullRequestsError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_list_pull_requests_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::list_pull_requests::ListPullRequestsOutput, crate::operation::list_pull_requests::ListPullRequestsError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::list_pull_requests::builders::ListPullRequestsOutputBuilder::default();
        output = crate::protocol_serde::shape_list_pull_requests::de_list_pull_requests(_response_body, output)
            .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::list_pull_requests_output_output_correct_errors(output)
            .build()
            .map_err(crate::operation::list_pull_requests::ListPullRequestsError::unhandled)?
    })
}

pub fn ser_list_pull_requests_input(
    input: &crate::operation::list_pull_requests::ListPullRequestsInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_list_pull_requests_input::ser_list_pull_requests_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_list_pull_requests(
    value: &[u8],
    mut builder: crate::operation::list_pull_requests::builders::ListPullRequestsOutputBuilder,
) -> ::std::result::Result<
    crate::operation::list_pull_requests::builders::ListPullRequestsOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "pullRequestIds" => {
                    builder = builder.set_pull_request_ids(crate::protocol_serde::shape_pull_request_id_list::de_pull_request_id_list(tokens)?);
                }
                "nextToken" => {
                    builder = builder.set_next_token(
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
    if tokens.next().is_some() {
        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "found more JSON tokens after completing parsing",
        ));
    }
    Ok(builder)
}

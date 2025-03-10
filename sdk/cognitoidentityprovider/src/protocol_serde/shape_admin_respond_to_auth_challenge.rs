// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_admin_respond_to_auth_challenge_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeOutput,
    crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AliasExistsException" => crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::AliasExistsException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AliasExistsExceptionBuilder::default();
                output = crate::protocol_serde::shape_alias_exists_exception::de_alias_exists_exception_json_err(_response_body, output)
                    .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "CodeMismatchException" => crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::CodeMismatchException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::CodeMismatchExceptionBuilder::default();
                output = crate::protocol_serde::shape_code_mismatch_exception::de_code_mismatch_exception_json_err(_response_body, output)
                    .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ExpiredCodeException" => crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::ExpiredCodeException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ExpiredCodeExceptionBuilder::default();
                output = crate::protocol_serde::shape_expired_code_exception::de_expired_code_exception_json_err(_response_body, output)
                    .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalErrorException" => crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::InternalErrorException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalErrorExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_error_exception::de_internal_error_exception_json_err(_response_body, output)
                    .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidEmailRoleAccessPolicyException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::InvalidEmailRoleAccessPolicyException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidEmailRoleAccessPolicyExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_email_role_access_policy_exception::de_invalid_email_role_access_policy_exception_json_err(_response_body, output).map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidLambdaResponseException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::InvalidLambdaResponseException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidLambdaResponseExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_lambda_response_exception::de_invalid_lambda_response_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidParameterException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::InvalidParameterException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidParameterExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_invalid_parameter_exception::de_invalid_parameter_exception_json_err(_response_body, output)
                            .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidPasswordException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::InvalidPasswordException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidPasswordExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_password_exception::de_invalid_password_exception_json_err(_response_body, output)
                        .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidSmsRoleAccessPolicyException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::InvalidSmsRoleAccessPolicyException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidSmsRoleAccessPolicyExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_invalid_sms_role_access_policy_exception::de_invalid_sms_role_access_policy_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidSmsRoleTrustRelationshipException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::InvalidSmsRoleTrustRelationshipException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidSmsRoleTrustRelationshipExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_sms_role_trust_relationship_exception::de_invalid_sms_role_trust_relationship_exception_json_err(_response_body, output).map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidUserPoolConfigurationException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::InvalidUserPoolConfigurationException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidUserPoolConfigurationExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_user_pool_configuration_exception::de_invalid_user_pool_configuration_exception_json_err(_response_body, output).map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "MFAMethodNotFoundException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::MfaMethodNotFoundException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::MfaMethodNotFoundExceptionBuilder::default();
                    output = crate::protocol_serde::shape_mfa_method_not_found_exception::de_mfa_method_not_found_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "NotAuthorizedException" => crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::NotAuthorizedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::NotAuthorizedExceptionBuilder::default();
                output = crate::protocol_serde::shape_not_authorized_exception::de_not_authorized_exception_json_err(_response_body, output)
                    .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "PasswordHistoryPolicyViolationException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::PasswordHistoryPolicyViolationException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::PasswordHistoryPolicyViolationExceptionBuilder::default();
                    output = crate::protocol_serde::shape_password_history_policy_violation_exception::de_password_history_policy_violation_exception_json_err(_response_body, output).map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "PasswordResetRequiredException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::PasswordResetRequiredException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::PasswordResetRequiredExceptionBuilder::default();
                    output = crate::protocol_serde::shape_password_reset_required_exception::de_password_reset_required_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "ResourceNotFoundException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::ResourceNotFoundException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                            .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "SoftwareTokenMFANotFoundException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::SoftwareTokenMfaNotFoundException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::SoftwareTokenMfaNotFoundExceptionBuilder::default();
                    output = crate::protocol_serde::shape_software_token_mfa_not_found_exception::de_software_token_mfa_not_found_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "TooManyRequestsException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::TooManyRequestsException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::TooManyRequestsExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_too_many_requests_exception::de_too_many_requests_exception_json_err(_response_body, output)
                            .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "UnexpectedLambdaException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::UnexpectedLambdaException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::UnexpectedLambdaExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_unexpected_lambda_exception::de_unexpected_lambda_exception_json_err(_response_body, output)
                            .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "UserLambdaValidationException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::UserLambdaValidationException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::UserLambdaValidationExceptionBuilder::default();
                    output = crate::protocol_serde::shape_user_lambda_validation_exception::de_user_lambda_validation_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "UserNotConfirmedException" => {
            crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::UserNotConfirmedException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::UserNotConfirmedExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_user_not_confirmed_exception::de_user_not_confirmed_exception_json_err(_response_body, output)
                            .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "UserNotFoundException" => crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::UserNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::UserNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_user_not_found_exception::de_user_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_admin_respond_to_auth_challenge_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeOutput,
    crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::admin_respond_to_auth_challenge::builders::AdminRespondToAuthChallengeOutputBuilder::default();
        output = crate::protocol_serde::shape_admin_respond_to_auth_challenge::de_admin_respond_to_auth_challenge(_response_body, output)
            .map_err(crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_admin_respond_to_auth_challenge_input(
    input: &crate::operation::admin_respond_to_auth_challenge::AdminRespondToAuthChallengeInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_admin_respond_to_auth_challenge_input::ser_admin_respond_to_auth_challenge_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_admin_respond_to_auth_challenge(
    value: &[u8],
    mut builder: crate::operation::admin_respond_to_auth_challenge::builders::AdminRespondToAuthChallengeOutputBuilder,
) -> ::std::result::Result<
    crate::operation::admin_respond_to_auth_challenge::builders::AdminRespondToAuthChallengeOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "ChallengeName" => {
                    builder = builder.set_challenge_name(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| crate::types::ChallengeNameType::from(u.as_ref())))
                            .transpose()?,
                    );
                }
                "Session" => {
                    builder = builder.set_session(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "ChallengeParameters" => {
                    builder = builder.set_challenge_parameters(crate::protocol_serde::shape_challenge_parameters_type::de_challenge_parameters_type(
                        tokens,
                    )?);
                }
                "AuthenticationResult" => {
                    builder = builder.set_authentication_result(
                        crate::protocol_serde::shape_authentication_result_type::de_authentication_result_type(tokens)?,
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

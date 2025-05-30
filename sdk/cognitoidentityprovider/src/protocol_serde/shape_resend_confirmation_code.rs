// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_resend_confirmation_code_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::resend_confirmation_code::ResendConfirmationCodeOutput,
    crate::operation::resend_confirmation_code::ResendConfirmationCodeError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => {
            return Err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled(
                generic,
            ))
        }
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "CodeDeliveryFailureException" => crate::operation::resend_confirmation_code::ResendConfirmationCodeError::CodeDeliveryFailureException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::CodeDeliveryFailureExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_code_delivery_failure_exception::de_code_delivery_failure_exception_json_err(_response_body, output)
                        .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ForbiddenException" => crate::operation::resend_confirmation_code::ResendConfirmationCodeError::ForbiddenException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ForbiddenExceptionBuilder::default();
                output = crate::protocol_serde::shape_forbidden_exception::de_forbidden_exception_json_err(_response_body, output)
                    .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalErrorException" => crate::operation::resend_confirmation_code::ResendConfirmationCodeError::InternalErrorException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalErrorExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_error_exception::de_internal_error_exception_json_err(_response_body, output)
                    .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidEmailRoleAccessPolicyException" => {
            crate::operation::resend_confirmation_code::ResendConfirmationCodeError::InvalidEmailRoleAccessPolicyException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidEmailRoleAccessPolicyExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_email_role_access_policy_exception::de_invalid_email_role_access_policy_exception_json_err(_response_body, output).map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
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
            crate::operation::resend_confirmation_code::ResendConfirmationCodeError::InvalidLambdaResponseException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidLambdaResponseExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_lambda_response_exception::de_invalid_lambda_response_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidParameterException" => crate::operation::resend_confirmation_code::ResendConfirmationCodeError::InvalidParameterException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidParameterExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_parameter_exception::de_invalid_parameter_exception_json_err(_response_body, output)
                    .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidSmsRoleAccessPolicyException" => {
            crate::operation::resend_confirmation_code::ResendConfirmationCodeError::InvalidSmsRoleAccessPolicyException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidSmsRoleAccessPolicyExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_invalid_sms_role_access_policy_exception::de_invalid_sms_role_access_policy_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
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
            crate::operation::resend_confirmation_code::ResendConfirmationCodeError::InvalidSmsRoleTrustRelationshipException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidSmsRoleTrustRelationshipExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_sms_role_trust_relationship_exception::de_invalid_sms_role_trust_relationship_exception_json_err(_response_body, output).map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "LimitExceededException" => crate::operation::resend_confirmation_code::ResendConfirmationCodeError::LimitExceededException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::LimitExceededExceptionBuilder::default();
                output = crate::protocol_serde::shape_limit_exceeded_exception::de_limit_exceeded_exception_json_err(_response_body, output)
                    .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "NotAuthorizedException" => crate::operation::resend_confirmation_code::ResendConfirmationCodeError::NotAuthorizedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::NotAuthorizedExceptionBuilder::default();
                output = crate::protocol_serde::shape_not_authorized_exception::de_not_authorized_exception_json_err(_response_body, output)
                    .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceNotFoundException" => crate::operation::resend_confirmation_code::ResendConfirmationCodeError::ResourceNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TooManyRequestsException" => crate::operation::resend_confirmation_code::ResendConfirmationCodeError::TooManyRequestsException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::TooManyRequestsExceptionBuilder::default();
                output = crate::protocol_serde::shape_too_many_requests_exception::de_too_many_requests_exception_json_err(_response_body, output)
                    .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "UnexpectedLambdaException" => crate::operation::resend_confirmation_code::ResendConfirmationCodeError::UnexpectedLambdaException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::UnexpectedLambdaExceptionBuilder::default();
                output = crate::protocol_serde::shape_unexpected_lambda_exception::de_unexpected_lambda_exception_json_err(_response_body, output)
                    .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "UserLambdaValidationException" => crate::operation::resend_confirmation_code::ResendConfirmationCodeError::UserLambdaValidationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::UserLambdaValidationExceptionBuilder::default();
                output = crate::protocol_serde::shape_user_lambda_validation_exception::de_user_lambda_validation_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "UserNotFoundException" => crate::operation::resend_confirmation_code::ResendConfirmationCodeError::UserNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::UserNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_user_not_found_exception::de_user_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::resend_confirmation_code::ResendConfirmationCodeError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_resend_confirmation_code_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::resend_confirmation_code::ResendConfirmationCodeOutput,
    crate::operation::resend_confirmation_code::ResendConfirmationCodeError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::resend_confirmation_code::builders::ResendConfirmationCodeOutputBuilder::default();
        output = crate::protocol_serde::shape_resend_confirmation_code::de_resend_confirmation_code(_response_body, output)
            .map_err(crate::operation::resend_confirmation_code::ResendConfirmationCodeError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_resend_confirmation_code_input(
    input: &crate::operation::resend_confirmation_code::ResendConfirmationCodeInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_resend_confirmation_code_input::ser_resend_confirmation_code_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_resend_confirmation_code(
    value: &[u8],
    mut builder: crate::operation::resend_confirmation_code::builders::ResendConfirmationCodeOutputBuilder,
) -> ::std::result::Result<
    crate::operation::resend_confirmation_code::builders::ResendConfirmationCodeOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "CodeDeliveryDetails" => {
                    builder = builder.set_code_delivery_details(
                        crate::protocol_serde::shape_code_delivery_details_type::de_code_delivery_details_type(tokens)?,
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

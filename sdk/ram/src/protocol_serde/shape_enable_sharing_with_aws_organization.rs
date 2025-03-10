// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_enable_sharing_with_aws_organization_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationOutput,
    crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "OperationNotPermittedException" => {
            crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::OperationNotPermittedException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::OperationNotPermittedExceptionBuilder::default();
                    output = crate::protocol_serde::shape_operation_not_permitted_exception::de_operation_not_permitted_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::unhandled)?;
                    let output = output.meta(generic);
                    crate::serde_util::operation_not_permitted_exception_correct_errors(output)
                        .build()
                        .map_err(crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::unhandled)?
                };
                tmp
            })
        }
        "ServerInternalException" => {
            crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::ServerInternalException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ServerInternalExceptionBuilder::default();
                    output = crate::protocol_serde::shape_server_internal_exception::de_server_internal_exception_json_err(_response_body, output)
                        .map_err(crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::unhandled)?;
                    let output = output.meta(generic);
                    crate::serde_util::server_internal_exception_correct_errors(output)
                        .build()
                        .map_err(crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::unhandled)?
                };
                tmp
            })
        }
        "ServiceUnavailableException" => {
            crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::ServiceUnavailableException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ServiceUnavailableExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_service_unavailable_exception::de_service_unavailable_exception_json_err(_response_body, output)
                            .map_err(crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::unhandled)?;
                    let output = output.meta(generic);
                    crate::serde_util::service_unavailable_exception_correct_errors(output)
                        .build()
                        .map_err(crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::unhandled)?
                };
                tmp
            })
        }
        _ => crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_enable_sharing_with_aws_organization_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationOutput,
    crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::enable_sharing_with_aws_organization::builders::EnableSharingWithAwsOrganizationOutputBuilder::default();
        output = crate::protocol_serde::shape_enable_sharing_with_aws_organization::de_enable_sharing_with_aws_organization(_response_body, output)
            .map_err(crate::operation::enable_sharing_with_aws_organization::EnableSharingWithAwsOrganizationError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub(crate) fn de_enable_sharing_with_aws_organization(
    value: &[u8],
    mut builder: crate::operation::enable_sharing_with_aws_organization::builders::EnableSharingWithAwsOrganizationOutputBuilder,
) -> ::std::result::Result<
    crate::operation::enable_sharing_with_aws_organization::builders::EnableSharingWithAwsOrganizationOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "returnValue" => {
                    builder = builder.set_return_value(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
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

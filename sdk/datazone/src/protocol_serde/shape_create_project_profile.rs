// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_create_project_profile_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::create_project_profile::CreateProjectProfileOutput,
    crate::operation::create_project_profile::CreateProjectProfileError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AccessDeniedException" => crate::operation::create_project_profile::CreateProjectProfileError::AccessDeniedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AccessDeniedExceptionBuilder::default();
                output = crate::protocol_serde::shape_access_denied_exception::de_access_denied_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::access_denied_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?
            };
            tmp
        }),
        "ConflictException" => crate::operation::create_project_profile::CreateProjectProfileError::ConflictException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ConflictExceptionBuilder::default();
                output = crate::protocol_serde::shape_conflict_exception::de_conflict_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::conflict_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?
            };
            tmp
        }),
        "InternalServerException" => crate::operation::create_project_profile::CreateProjectProfileError::InternalServerException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_server_exception::de_internal_server_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::internal_server_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?
            };
            tmp
        }),
        "ResourceNotFoundException" => crate::operation::create_project_profile::CreateProjectProfileError::ResourceNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::resource_not_found_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?
            };
            tmp
        }),
        "ServiceQuotaExceededException" => crate::operation::create_project_profile::CreateProjectProfileError::ServiceQuotaExceededException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ServiceQuotaExceededExceptionBuilder::default();
                output = crate::protocol_serde::shape_service_quota_exceeded_exception::de_service_quota_exceeded_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::service_quota_exceeded_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?
            };
            tmp
        }),
        "ThrottlingException" => crate::operation::create_project_profile::CreateProjectProfileError::ThrottlingException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingExceptionBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::throttling_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?
            };
            tmp
        }),
        "ValidationException" => crate::operation::create_project_profile::CreateProjectProfileError::ValidationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationExceptionBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::validation_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?
            };
            tmp
        }),
        "UnauthorizedException" => crate::operation::create_project_profile::CreateProjectProfileError::UnauthorizedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::UnauthorizedExceptionBuilder::default();
                output = crate::protocol_serde::shape_unauthorized_exception::de_unauthorized_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::unauthorized_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?
            };
            tmp
        }),
        _ => crate::operation::create_project_profile::CreateProjectProfileError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_create_project_profile_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::create_project_profile::CreateProjectProfileOutput,
    crate::operation::create_project_profile::CreateProjectProfileError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::create_project_profile::builders::CreateProjectProfileOutputBuilder::default();
        output = crate::protocol_serde::shape_create_project_profile::de_create_project_profile(_response_body, output)
            .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::create_project_profile_output_output_correct_errors(output)
            .build()
            .map_err(crate::operation::create_project_profile::CreateProjectProfileError::unhandled)?
    })
}

pub fn ser_create_project_profile_input(
    input: &crate::operation::create_project_profile::CreateProjectProfileInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_create_project_profile_input::ser_create_project_profile_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_create_project_profile(
    value: &[u8],
    mut builder: crate::operation::create_project_profile::builders::CreateProjectProfileOutputBuilder,
) -> ::std::result::Result<
    crate::operation::create_project_profile::builders::CreateProjectProfileOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "createdAt" => {
                    builder = builder.set_created_at(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                        tokens.next(),
                        ::aws_smithy_types::date_time::Format::DateTimeWithOffset,
                    )?);
                }
                "createdBy" => {
                    builder = builder.set_created_by(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "description" => {
                    builder = builder.set_description(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "domainId" => {
                    builder = builder.set_domain_id(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "domainUnitId" => {
                    builder = builder.set_domain_unit_id(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "environmentConfigurations" => {
                    builder = builder.set_environment_configurations(
                        crate::protocol_serde::shape_environment_configurations_list::de_environment_configurations_list(tokens)?,
                    );
                }
                "id" => {
                    builder = builder.set_id(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "lastUpdatedAt" => {
                    builder = builder.set_last_updated_at(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                        tokens.next(),
                        ::aws_smithy_types::date_time::Format::DateTimeWithOffset,
                    )?);
                }
                "name" => {
                    builder = builder.set_name(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "status" => {
                    builder = builder.set_status(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| crate::types::Status::from(u.as_ref())))
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

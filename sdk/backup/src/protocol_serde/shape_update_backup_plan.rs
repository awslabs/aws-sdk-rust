// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_update_backup_plan_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::update_backup_plan::UpdateBackupPlanOutput, crate::operation::update_backup_plan::UpdateBackupPlanError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::update_backup_plan::UpdateBackupPlanError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::update_backup_plan::UpdateBackupPlanError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "InvalidParameterValueException" => crate::operation::update_backup_plan::UpdateBackupPlanError::InvalidParameterValueException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidParameterValueExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_parameter_value_exception::de_invalid_parameter_value_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::update_backup_plan::UpdateBackupPlanError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "MissingParameterValueException" => crate::operation::update_backup_plan::UpdateBackupPlanError::MissingParameterValueException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::MissingParameterValueExceptionBuilder::default();
                output = crate::protocol_serde::shape_missing_parameter_value_exception::de_missing_parameter_value_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::update_backup_plan::UpdateBackupPlanError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceNotFoundException" => crate::operation::update_backup_plan::UpdateBackupPlanError::ResourceNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::update_backup_plan::UpdateBackupPlanError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ServiceUnavailableException" => crate::operation::update_backup_plan::UpdateBackupPlanError::ServiceUnavailableException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ServiceUnavailableExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_service_unavailable_exception::de_service_unavailable_exception_json_err(_response_body, output)
                        .map_err(crate::operation::update_backup_plan::UpdateBackupPlanError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::update_backup_plan::UpdateBackupPlanError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_update_backup_plan_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::update_backup_plan::UpdateBackupPlanOutput, crate::operation::update_backup_plan::UpdateBackupPlanError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::update_backup_plan::builders::UpdateBackupPlanOutputBuilder::default();
        output = crate::protocol_serde::shape_update_backup_plan::de_update_backup_plan(_response_body, output)
            .map_err(crate::operation::update_backup_plan::UpdateBackupPlanError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_update_backup_plan_input(
    input: &crate::operation::update_backup_plan::UpdateBackupPlanInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_update_backup_plan_input::ser_update_backup_plan_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_update_backup_plan(
    value: &[u8],
    mut builder: crate::operation::update_backup_plan::builders::UpdateBackupPlanOutputBuilder,
) -> ::std::result::Result<
    crate::operation::update_backup_plan::builders::UpdateBackupPlanOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "AdvancedBackupSettings" => {
                    builder = builder.set_advanced_backup_settings(
                        crate::protocol_serde::shape_advanced_backup_settings::de_advanced_backup_settings(tokens)?,
                    );
                }
                "BackupPlanArn" => {
                    builder = builder.set_backup_plan_arn(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "BackupPlanId" => {
                    builder = builder.set_backup_plan_id(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "CreationDate" => {
                    builder = builder.set_creation_date(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                        tokens.next(),
                        ::aws_smithy_types::date_time::Format::EpochSeconds,
                    )?);
                }
                "VersionId" => {
                    builder = builder.set_version_id(
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

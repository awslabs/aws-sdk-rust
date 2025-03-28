// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_export_vector_enrichment_job_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobOutput,
    crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => {
            return Err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled(
                generic,
            ))
        }
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AccessDeniedException" => crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::AccessDeniedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AccessDeniedExceptionBuilder::default();
                output = crate::protocol_serde::shape_access_denied_exception::de_access_denied_exception_json_err(_response_body, output)
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::access_denied_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?
            };
            tmp
        }),
        "ConflictException" => crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::ConflictException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ConflictExceptionBuilder::default();
                output = crate::protocol_serde::shape_conflict_exception::de_conflict_exception_json_err(_response_body, output)
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::conflict_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?
            };
            tmp
        }),
        "InternalServerException" => crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::InternalServerException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_server_exception::de_internal_server_exception_json_err(_response_body, output)
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::internal_server_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?
            };
            tmp
        }),
        "ResourceNotFoundException" => crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::ResourceNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::resource_not_found_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?
            };
            tmp
        }),
        "ServiceQuotaExceededException" => {
            crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::ServiceQuotaExceededException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ServiceQuotaExceededExceptionBuilder::default();
                    output = crate::protocol_serde::shape_service_quota_exceeded_exception::de_service_quota_exceeded_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?;
                    let output = output.meta(generic);
                    crate::serde_util::service_quota_exceeded_exception_correct_errors(output)
                        .build()
                        .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?
                };
                tmp
            })
        }
        "ThrottlingException" => crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::ThrottlingException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingExceptionBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(_response_body, output)
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::throttling_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?
            };
            tmp
        }),
        "ValidationException" => crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::ValidationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationExceptionBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_json_err(_response_body, output)
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::validation_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?
            };
            tmp
        }),
        _ => crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_export_vector_enrichment_job_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobOutput,
    crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::export_vector_enrichment_job::builders::ExportVectorEnrichmentJobOutputBuilder::default();
        output = crate::protocol_serde::shape_export_vector_enrichment_job::de_export_vector_enrichment_job(_response_body, output)
            .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::export_vector_enrichment_job_output_output_correct_errors(output)
            .build()
            .map_err(crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobError::unhandled)?
    })
}

pub fn ser_export_vector_enrichment_job_input(
    input: &crate::operation::export_vector_enrichment_job::ExportVectorEnrichmentJobInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_export_vector_enrichment_job_input::ser_export_vector_enrichment_job_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_export_vector_enrichment_job(
    value: &[u8],
    mut builder: crate::operation::export_vector_enrichment_job::builders::ExportVectorEnrichmentJobOutputBuilder,
) -> ::std::result::Result<
    crate::operation::export_vector_enrichment_job::builders::ExportVectorEnrichmentJobOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "Arn" => {
                    builder = builder.set_arn(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "CreationTime" => {
                    builder = builder.set_creation_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                        tokens.next(),
                        ::aws_smithy_types::date_time::Format::DateTimeWithOffset,
                    )?);
                }
                "ExecutionRoleArn" => {
                    builder = builder.set_execution_role_arn(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "ExportStatus" => {
                    builder = builder.set_export_status(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| crate::types::VectorEnrichmentJobExportStatus::from(u.as_ref())))
                            .transpose()?,
                    );
                }
                "OutputConfig" => {
                    builder = builder.set_output_config(
                        crate::protocol_serde::shape_export_vector_enrichment_job_output_config::de_export_vector_enrichment_job_output_config(
                            tokens,
                        )?,
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

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_start_replication_task_assessment_run_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunOutput,
    crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AccessDeniedFault" => crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::AccessDeniedFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AccessDeniedFaultBuilder::default();
                output = crate::protocol_serde::shape_access_denied_fault::de_access_denied_fault_json_err(_response_body, output)
                    .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidResourceStateFault" => {
            crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::InvalidResourceStateFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidResourceStateFaultBuilder::default();
                    output =
                        crate::protocol_serde::shape_invalid_resource_state_fault::de_invalid_resource_state_fault_json_err(_response_body, output)
                            .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "KMSAccessDeniedFault" => {
            crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::KmsAccessDeniedFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::KmsAccessDeniedFaultBuilder::default();
                    output = crate::protocol_serde::shape_kms_access_denied_fault::de_kms_access_denied_fault_json_err(_response_body, output)
                        .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "KMSDisabledFault" => crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::KmsDisabledFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::KmsDisabledFaultBuilder::default();
                output = crate::protocol_serde::shape_kms_disabled_fault::de_kms_disabled_fault_json_err(_response_body, output)
                    .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "KMSFault" => crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::KmsFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::KmsFaultBuilder::default();
                output = crate::protocol_serde::shape_kms_fault::de_kms_fault_json_err(_response_body, output)
                    .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "KMSInvalidStateFault" => {
            crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::KmsInvalidStateFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::KmsInvalidStateFaultBuilder::default();
                    output = crate::protocol_serde::shape_kms_invalid_state_fault::de_kms_invalid_state_fault_json_err(_response_body, output)
                        .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "KMSKeyNotAccessibleFault" => {
            crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::KmsKeyNotAccessibleFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::KmsKeyNotAccessibleFaultBuilder::default();
                    output =
                        crate::protocol_serde::shape_kms_key_not_accessible_fault::de_kms_key_not_accessible_fault_json_err(_response_body, output)
                            .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "KMSNotFoundFault" => crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::KmsNotFoundFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::KmsNotFoundFaultBuilder::default();
                output = crate::protocol_serde::shape_kms_not_found_fault::de_kms_not_found_fault_json_err(_response_body, output)
                    .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceAlreadyExistsFault" => {
            crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::ResourceAlreadyExistsFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ResourceAlreadyExistsFaultBuilder::default();
                    output =
                        crate::protocol_serde::shape_resource_already_exists_fault::de_resource_already_exists_fault_json_err(_response_body, output)
                            .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "ResourceNotFoundFault" => {
            crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::ResourceNotFoundFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ResourceNotFoundFaultBuilder::default();
                    output = crate::protocol_serde::shape_resource_not_found_fault::de_resource_not_found_fault_json_err(_response_body, output)
                        .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "S3AccessDeniedFault" => {
            crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::S3AccessDeniedFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::S3AccessDeniedFaultBuilder::default();
                    output = crate::protocol_serde::shape_s3_access_denied_fault::de_s3_access_denied_fault_json_err(_response_body, output)
                        .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "S3ResourceNotFoundFault" => {
            crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::S3ResourceNotFoundFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::S3ResourceNotFoundFaultBuilder::default();
                    output =
                        crate::protocol_serde::shape_s3_resource_not_found_fault::de_s3_resource_not_found_fault_json_err(_response_body, output)
                            .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        _ => crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_start_replication_task_assessment_run_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunOutput,
    crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::start_replication_task_assessment_run::builders::StartReplicationTaskAssessmentRunOutputBuilder::default();
        output = crate::protocol_serde::shape_start_replication_task_assessment_run::de_start_replication_task_assessment_run(_response_body, output)
            .map_err(crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_start_replication_task_assessment_run_input(
    input: &crate::operation::start_replication_task_assessment_run::StartReplicationTaskAssessmentRunInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_start_replication_task_assessment_run_input::ser_start_replication_task_assessment_run_input_input(
        &mut object,
        input,
    )?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_start_replication_task_assessment_run(
    value: &[u8],
    mut builder: crate::operation::start_replication_task_assessment_run::builders::StartReplicationTaskAssessmentRunOutputBuilder,
) -> ::std::result::Result<
    crate::operation::start_replication_task_assessment_run::builders::StartReplicationTaskAssessmentRunOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "ReplicationTaskAssessmentRun" => {
                    builder = builder.set_replication_task_assessment_run(
                        crate::protocol_serde::shape_replication_task_assessment_run::de_replication_task_assessment_run(tokens)?,
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

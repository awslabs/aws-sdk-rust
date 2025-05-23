// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_create_table_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::create_table::CreateTableOutput, crate::operation::create_table::CreateTableError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::create_table::CreateTableError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::create_table::CreateTableError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AlreadyExistsException" => crate::operation::create_table::CreateTableError::AlreadyExistsException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AlreadyExistsExceptionBuilder::default();
                output = crate::protocol_serde::shape_already_exists_exception::de_already_exists_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_table::CreateTableError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ConcurrentModificationException" => crate::operation::create_table::CreateTableError::ConcurrentModificationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ConcurrentModificationExceptionBuilder::default();
                output = crate::protocol_serde::shape_concurrent_modification_exception::de_concurrent_modification_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::create_table::CreateTableError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "EntityNotFoundException" => crate::operation::create_table::CreateTableError::EntityNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::EntityNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_entity_not_found_exception::de_entity_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_table::CreateTableError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "FederationSourceException" => crate::operation::create_table::CreateTableError::FederationSourceException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::FederationSourceExceptionBuilder::default();
                output = crate::protocol_serde::shape_federation_source_exception::de_federation_source_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_table::CreateTableError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "FederationSourceRetryableException" => crate::operation::create_table::CreateTableError::FederationSourceRetryableException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::FederationSourceRetryableExceptionBuilder::default();
                output = crate::protocol_serde::shape_federation_source_retryable_exception::de_federation_source_retryable_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::create_table::CreateTableError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "GlueEncryptionException" => crate::operation::create_table::CreateTableError::GlueEncryptionException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::GlueEncryptionExceptionBuilder::default();
                output = crate::protocol_serde::shape_glue_encryption_exception::de_glue_encryption_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_table::CreateTableError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalServiceException" => crate::operation::create_table::CreateTableError::InternalServiceException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServiceExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_service_exception::de_internal_service_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_table::CreateTableError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidInputException" => crate::operation::create_table::CreateTableError::InvalidInputException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidInputExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_input_exception::de_invalid_input_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_table::CreateTableError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "OperationTimeoutException" => crate::operation::create_table::CreateTableError::OperationTimeoutException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::OperationTimeoutExceptionBuilder::default();
                output = crate::protocol_serde::shape_operation_timeout_exception::de_operation_timeout_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_table::CreateTableError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceNotReadyException" => crate::operation::create_table::CreateTableError::ResourceNotReadyException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotReadyExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_ready_exception::de_resource_not_ready_exception_json_err(_response_body, output)
                    .map_err(crate::operation::create_table::CreateTableError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceNumberLimitExceededException" => crate::operation::create_table::CreateTableError::ResourceNumberLimitExceededException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNumberLimitExceededExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_number_limit_exceeded_exception::de_resource_number_limit_exceeded_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::create_table::CreateTableError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::create_table::CreateTableError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_create_table_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::create_table::CreateTableOutput, crate::operation::create_table::CreateTableError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::create_table::builders::CreateTableOutputBuilder::default();
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_create_table_input(
    input: &crate::operation::create_table::CreateTableInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_create_table_input::ser_create_table_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_transact_write_items_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::transact_write_items::TransactWriteItemsOutput,
    crate::operation::transact_write_items::TransactWriteItemsError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::transact_write_items::TransactWriteItemsError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::transact_write_items::TransactWriteItemsError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "IdempotentParameterMismatchException" => {
            crate::operation::transact_write_items::TransactWriteItemsError::IdempotentParameterMismatchException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::IdempotentParameterMismatchExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_idempotent_parameter_mismatch_exception::de_idempotent_parameter_mismatch_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::transact_write_items::TransactWriteItemsError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InternalServerError" => crate::operation::transact_write_items::TransactWriteItemsError::InternalServerError({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerErrorBuilder::default();
                output = crate::protocol_serde::shape_internal_server_error::de_internal_server_error_json_err(_response_body, output)
                    .map_err(crate::operation::transact_write_items::TransactWriteItemsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidEndpointException" => crate::operation::transact_write_items::TransactWriteItemsError::InvalidEndpointException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidEndpointExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_endpoint_exception::de_invalid_endpoint_exception_json_err(_response_body, output)
                    .map_err(crate::operation::transact_write_items::TransactWriteItemsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ProvisionedThroughputExceededException" => {
            crate::operation::transact_write_items::TransactWriteItemsError::ProvisionedThroughputExceededException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ProvisionedThroughputExceededExceptionBuilder::default();
                    output = crate::protocol_serde::shape_provisioned_throughput_exceeded_exception::de_provisioned_throughput_exceeded_exception_json_err(_response_body, output).map_err(crate::operation::transact_write_items::TransactWriteItemsError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "RequestLimitExceeded" => crate::operation::transact_write_items::TransactWriteItemsError::RequestLimitExceeded({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::RequestLimitExceededBuilder::default();
                output = crate::protocol_serde::shape_request_limit_exceeded::de_request_limit_exceeded_json_err(_response_body, output)
                    .map_err(crate::operation::transact_write_items::TransactWriteItemsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceNotFoundException" => crate::operation::transact_write_items::TransactWriteItemsError::ResourceNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::transact_write_items::TransactWriteItemsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TransactionCanceledException" => crate::operation::transact_write_items::TransactWriteItemsError::TransactionCanceledException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::TransactionCanceledExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_transaction_canceled_exception::de_transaction_canceled_exception_json_err(_response_body, output)
                        .map_err(crate::operation::transact_write_items::TransactWriteItemsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TransactionInProgressException" => crate::operation::transact_write_items::TransactWriteItemsError::TransactionInProgressException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::TransactionInProgressExceptionBuilder::default();
                output = crate::protocol_serde::shape_transaction_in_progress_exception::de_transaction_in_progress_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::transact_write_items::TransactWriteItemsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::transact_write_items::TransactWriteItemsError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_transact_write_items_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::transact_write_items::TransactWriteItemsOutput,
    crate::operation::transact_write_items::TransactWriteItemsError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::transact_write_items::builders::TransactWriteItemsOutputBuilder::default();
        output = crate::protocol_serde::shape_transact_write_items::de_transact_write_items(_response_body, output)
            .map_err(crate::operation::transact_write_items::TransactWriteItemsError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_transact_write_items_input(
    input: &crate::operation::transact_write_items::TransactWriteItemsInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_transact_write_items_input::ser_transact_write_items_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_transact_write_items(
    value: &[u8],
    mut builder: crate::operation::transact_write_items::builders::TransactWriteItemsOutputBuilder,
) -> ::std::result::Result<
    crate::operation::transact_write_items::builders::TransactWriteItemsOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "ConsumedCapacity" => {
                    builder = builder.set_consumed_capacity(crate::protocol_serde::shape_consumed_capacity_multiple::de_consumed_capacity_multiple(
                        tokens,
                    )?);
                }
                "ItemCollectionMetrics" => {
                    builder = builder.set_item_collection_metrics(
                        crate::protocol_serde::shape_item_collection_metrics_per_table::de_item_collection_metrics_per_table(tokens)?,
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

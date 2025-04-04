// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_get_statement_result_v2_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::get_statement_result_v2::GetStatementResultV2Output,
    crate::operation::get_statement_result_v2::GetStatementResultV2Error,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::get_statement_result_v2::GetStatementResultV2Error::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::get_statement_result_v2::GetStatementResultV2Error::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "InternalServerException" => crate::operation::get_statement_result_v2::GetStatementResultV2Error::InternalServerException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_server_exception::de_internal_server_exception_json_err(_response_body, output)
                    .map_err(crate::operation::get_statement_result_v2::GetStatementResultV2Error::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::internal_server_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::get_statement_result_v2::GetStatementResultV2Error::unhandled)?
            };
            tmp
        }),
        "ResourceNotFoundException" => crate::operation::get_statement_result_v2::GetStatementResultV2Error::ResourceNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_exception::de_resource_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::get_statement_result_v2::GetStatementResultV2Error::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::resource_not_found_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::get_statement_result_v2::GetStatementResultV2Error::unhandled)?
            };
            tmp
        }),
        "ValidationException" => crate::operation::get_statement_result_v2::GetStatementResultV2Error::ValidationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationExceptionBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_json_err(_response_body, output)
                    .map_err(crate::operation::get_statement_result_v2::GetStatementResultV2Error::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::get_statement_result_v2::GetStatementResultV2Error::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_get_statement_result_v2_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::get_statement_result_v2::GetStatementResultV2Output,
    crate::operation::get_statement_result_v2::GetStatementResultV2Error,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::get_statement_result_v2::builders::GetStatementResultV2OutputBuilder::default();
        output = crate::protocol_serde::shape_get_statement_result_v2::de_get_statement_result_v2(_response_body, output)
            .map_err(crate::operation::get_statement_result_v2::GetStatementResultV2Error::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::get_statement_result_v2_output_output_correct_errors(output)
            .build()
            .map_err(crate::operation::get_statement_result_v2::GetStatementResultV2Error::unhandled)?
    })
}

pub fn ser_get_statement_result_v2_input(
    input: &crate::operation::get_statement_result_v2::GetStatementResultV2Input,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_get_statement_result_v2_input::ser_get_statement_result_v2_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_get_statement_result_v2(
    value: &[u8],
    mut builder: crate::operation::get_statement_result_v2::builders::GetStatementResultV2OutputBuilder,
) -> ::std::result::Result<
    crate::operation::get_statement_result_v2::builders::GetStatementResultV2OutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "Records" => {
                    builder = builder.set_records(crate::protocol_serde::shape_formatted_sql_records::de_formatted_sql_records(tokens)?);
                }
                "ColumnMetadata" => {
                    builder = builder.set_column_metadata(crate::protocol_serde::shape_column_metadata_list::de_column_metadata_list(tokens)?);
                }
                "TotalNumRows" => {
                    builder = builder.set_total_num_rows(
                        ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                            .map(i64::try_from)
                            .transpose()?,
                    );
                }
                "ResultFormat" => {
                    builder = builder.set_result_format(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| crate::types::ResultFormatString::from(u.as_ref())))
                            .transpose()?,
                    );
                }
                "NextToken" => {
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

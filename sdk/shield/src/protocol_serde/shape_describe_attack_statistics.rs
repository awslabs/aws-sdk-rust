// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_attack_statistics_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::describe_attack_statistics::DescribeAttackStatisticsOutput,
    crate::operation::describe_attack_statistics::DescribeAttackStatisticsError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::describe_attack_statistics::DescribeAttackStatisticsError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => {
            return Err(crate::operation::describe_attack_statistics::DescribeAttackStatisticsError::unhandled(
                generic,
            ))
        }
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "InternalErrorException" => crate::operation::describe_attack_statistics::DescribeAttackStatisticsError::InternalErrorException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalErrorExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_error_exception::de_internal_error_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_attack_statistics::DescribeAttackStatisticsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::describe_attack_statistics::DescribeAttackStatisticsError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_attack_statistics_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::describe_attack_statistics::DescribeAttackStatisticsOutput,
    crate::operation::describe_attack_statistics::DescribeAttackStatisticsError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::describe_attack_statistics::builders::DescribeAttackStatisticsOutputBuilder::default();
        output = crate::protocol_serde::shape_describe_attack_statistics::de_describe_attack_statistics(_response_body, output)
            .map_err(crate::operation::describe_attack_statistics::DescribeAttackStatisticsError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::describe_attack_statistics_output_output_correct_errors(output)
            .build()
            .map_err(crate::operation::describe_attack_statistics::DescribeAttackStatisticsError::unhandled)?
    })
}

pub fn ser_describe_attack_statistics_input(
    _input: &crate::operation::describe_attack_statistics::DescribeAttackStatisticsInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    Ok(::aws_smithy_types::body::SdkBody::from("{}"))
}

pub(crate) fn de_describe_attack_statistics(
    value: &[u8],
    mut builder: crate::operation::describe_attack_statistics::builders::DescribeAttackStatisticsOutputBuilder,
) -> ::std::result::Result<
    crate::operation::describe_attack_statistics::builders::DescribeAttackStatisticsOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "TimeRange" => {
                    builder = builder.set_time_range(crate::protocol_serde::shape_time_range::de_time_range(tokens)?);
                }
                "DataItems" => {
                    builder = builder.set_data_items(crate::protocol_serde::shape_attack_statistics_data_list::de_attack_statistics_data_list(
                        tokens,
                    )?);
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

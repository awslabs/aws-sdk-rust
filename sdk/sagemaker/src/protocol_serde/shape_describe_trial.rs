// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_trial_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::describe_trial::DescribeTrialOutput, crate::operation::describe_trial::DescribeTrialError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::describe_trial::DescribeTrialError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::describe_trial::DescribeTrialError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "ResourceNotFound" => crate::operation::describe_trial::DescribeTrialError::ResourceNotFound({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found::de_resource_not_found_json_err(_response_body, output)
                    .map_err(crate::operation::describe_trial::DescribeTrialError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::describe_trial::DescribeTrialError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_trial_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::describe_trial::DescribeTrialOutput, crate::operation::describe_trial::DescribeTrialError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::describe_trial::builders::DescribeTrialOutputBuilder::default();
        output = crate::protocol_serde::shape_describe_trial::de_describe_trial(_response_body, output)
            .map_err(crate::operation::describe_trial::DescribeTrialError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_describe_trial_input(
    input: &crate::operation::describe_trial::DescribeTrialInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_describe_trial_input::ser_describe_trial_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_describe_trial(
    value: &[u8],
    mut builder: crate::operation::describe_trial::builders::DescribeTrialOutputBuilder,
) -> ::std::result::Result<
    crate::operation::describe_trial::builders::DescribeTrialOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "TrialName" => {
                    builder = builder.set_trial_name(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "TrialArn" => {
                    builder = builder.set_trial_arn(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "DisplayName" => {
                    builder = builder.set_display_name(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "ExperimentName" => {
                    builder = builder.set_experiment_name(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "Source" => {
                    builder = builder.set_source(crate::protocol_serde::shape_trial_source::de_trial_source(tokens)?);
                }
                "CreationTime" => {
                    builder = builder.set_creation_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                        tokens.next(),
                        ::aws_smithy_types::date_time::Format::EpochSeconds,
                    )?);
                }
                "CreatedBy" => {
                    builder = builder.set_created_by(crate::protocol_serde::shape_user_context::de_user_context(tokens)?);
                }
                "LastModifiedTime" => {
                    builder = builder.set_last_modified_time(::aws_smithy_json::deserialize::token::expect_timestamp_or_null(
                        tokens.next(),
                        ::aws_smithy_types::date_time::Format::EpochSeconds,
                    )?);
                }
                "LastModifiedBy" => {
                    builder = builder.set_last_modified_by(crate::protocol_serde::shape_user_context::de_user_context(tokens)?);
                }
                "MetadataProperties" => {
                    builder = builder.set_metadata_properties(crate::protocol_serde::shape_metadata_properties::de_metadata_properties(tokens)?);
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

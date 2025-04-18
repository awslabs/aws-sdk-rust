// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_list_hapgs_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::list_hapgs::ListHapgsOutput, crate::operation::list_hapgs::ListHapgsError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::list_hapgs::ListHapgsError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::list_hapgs::ListHapgsError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "CloudHsmInternalException" => crate::operation::list_hapgs::ListHapgsError::CloudHsmInternalException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::CloudHsmInternalExceptionBuilder::default();
                output = crate::protocol_serde::shape_cloud_hsm_internal_exception::de_cloud_hsm_internal_exception_json_err(_response_body, output)
                    .map_err(crate::operation::list_hapgs::ListHapgsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "CloudHsmServiceException" => crate::operation::list_hapgs::ListHapgsError::CloudHsmServiceException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::CloudHsmServiceExceptionBuilder::default();
                output = crate::protocol_serde::shape_cloud_hsm_service_exception::de_cloud_hsm_service_exception_json_err(_response_body, output)
                    .map_err(crate::operation::list_hapgs::ListHapgsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidRequestException" => crate::operation::list_hapgs::ListHapgsError::InvalidRequestException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidRequestExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_request_exception::de_invalid_request_exception_json_err(_response_body, output)
                    .map_err(crate::operation::list_hapgs::ListHapgsError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::list_hapgs::ListHapgsError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_list_hapgs_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::list_hapgs::ListHapgsOutput, crate::operation::list_hapgs::ListHapgsError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::list_hapgs::builders::ListHapgsOutputBuilder::default();
        output = crate::protocol_serde::shape_list_hapgs::de_list_hapgs(_response_body, output)
            .map_err(crate::operation::list_hapgs::ListHapgsError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        crate::serde_util::list_hapgs_output_output_correct_errors(output)
            .build()
            .map_err(crate::operation::list_hapgs::ListHapgsError::unhandled)?
    })
}

pub fn ser_list_hapgs_input(
    input: &crate::operation::list_hapgs::ListHapgsInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_list_hapgs_input::ser_list_hapgs_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_list_hapgs(
    value: &[u8],
    mut builder: crate::operation::list_hapgs::builders::ListHapgsOutputBuilder,
) -> ::std::result::Result<crate::operation::list_hapgs::builders::ListHapgsOutputBuilder, ::aws_smithy_json::deserialize::error::DeserializeError> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "HapgList" => {
                    builder = builder.set_hapg_list(crate::protocol_serde::shape_hapg_list::de_hapg_list(tokens)?);
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

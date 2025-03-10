// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_execute_query_http_response(
    response: &mut ::aws_smithy_runtime_api::http::Response,
) -> std::result::Result<crate::operation::execute_query::ExecuteQueryOutput, crate::operation::execute_query::ExecuteQueryError> {
    let mut _response_body = ::aws_smithy_types::body::SdkBody::taken();
    std::mem::swap(&mut _response_body, response.body_mut());
    let _response_body = &mut _response_body;

    let _response_status = response.status().as_u16();
    let _response_headers = response.headers();
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::execute_query::builders::ExecuteQueryOutputBuilder::default();
        output = output.set_payload(Some(crate::protocol_serde::shape_execute_query_output::de_payload_payload(
            _response_body,
        )?));
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_execute_query_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::execute_query::ExecuteQueryOutput, crate::operation::execute_query::ExecuteQueryError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::execute_query::ExecuteQueryError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AccessDeniedException" => crate::operation::execute_query::ExecuteQueryError::AccessDeniedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AccessDeniedExceptionBuilder::default();
                output = crate::protocol_serde::shape_access_denied_exception::de_access_denied_exception_json_err(_response_body, output)
                    .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::access_denied_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?
            };
            tmp
        }),
        "ConflictException" => crate::operation::execute_query::ExecuteQueryError::ConflictException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ConflictExceptionBuilder::default();
                output = crate::protocol_serde::shape_conflict_exception::de_conflict_exception_json_err(_response_body, output)
                    .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::conflict_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?
            };
            tmp
        }),
        "InternalServerException" => crate::operation::execute_query::ExecuteQueryError::InternalServerException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerExceptionBuilder::default();
                output = crate::protocol_serde::shape_internal_server_exception::de_internal_server_exception_json_err(_response_body, output)
                    .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::internal_server_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?
            };
            tmp
        }),
        "ThrottlingException" => crate::operation::execute_query::ExecuteQueryError::ThrottlingException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingExceptionBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(_response_body, output)
                    .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::throttling_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?
            };
            tmp
        }),
        "UnprocessableException" => crate::operation::execute_query::ExecuteQueryError::UnprocessableException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::UnprocessableExceptionBuilder::default();
                output = crate::protocol_serde::shape_unprocessable_exception::de_unprocessable_exception_json_err(_response_body, output)
                    .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::unprocessable_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?
            };
            tmp
        }),
        "ValidationException" => crate::operation::execute_query::ExecuteQueryError::ValidationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ValidationExceptionBuilder::default();
                output = crate::protocol_serde::shape_validation_exception::de_validation_exception_json_err(_response_body, output)
                    .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?;
                let output = output.meta(generic);
                crate::serde_util::validation_exception_correct_errors(output)
                    .build()
                    .map_err(crate::operation::execute_query::ExecuteQueryError::unhandled)?
            };
            tmp
        }),
        _ => crate::operation::execute_query::ExecuteQueryError::generic(generic),
    })
}

pub fn ser_execute_query_headers(
    input: &crate::operation::execute_query::ExecuteQueryInput,
    mut builder: ::http::request::Builder,
) -> std::result::Result<::http::request::Builder, ::aws_smithy_types::error::operation::BuildError> {
    if let ::std::option::Option::Some(inner_1) = &input.graph_identifier {
        let formatted_2 = inner_1.as_str();
        let header_value = formatted_2;
        let header_value: ::http::HeaderValue = header_value.parse().map_err(|err| {
            ::aws_smithy_types::error::operation::BuildError::invalid_field(
                "graph_identifier",
                format!("`{}` cannot be used as a header value: {}", &header_value, err),
            )
        })?;
        builder = builder.header("graphIdentifier", header_value);
    }
    Ok(builder)
}

pub fn ser_execute_query_input(
    input: &crate::operation::execute_query::ExecuteQueryInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_execute_query_input::ser_execute_query_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

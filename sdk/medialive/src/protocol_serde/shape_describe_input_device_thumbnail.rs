// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_input_device_thumbnail_http_response(
    response: &mut ::aws_smithy_runtime_api::http::Response,
) -> std::result::Result<
    crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailOutput,
    crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError,
> {
    let mut _response_body = ::aws_smithy_types::body::SdkBody::taken();
    std::mem::swap(&mut _response_body, response.body_mut());
    let _response_body = &mut _response_body;

    let _response_status = response.status().as_u16();
    let _response_headers = response.headers();
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::describe_input_device_thumbnail::builders::DescribeInputDeviceThumbnailOutputBuilder::default();
        output = output.set_body(Some(
            crate::protocol_serde::shape_describe_input_device_thumbnail_output::de_body_payload(_response_body)?,
        ));
        output = output.set_content_length(
            crate::protocol_serde::shape_describe_input_device_thumbnail_output::de_content_length_header(_response_headers).map_err(|_| {
                crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled(
                    "Failed to parse ContentLength from header `Content-Length",
                )
            })?,
        );
        output = output.set_content_type(
            crate::protocol_serde::shape_describe_input_device_thumbnail_output::de_content_type_header(_response_headers).map_err(|_| {
                crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled(
                    "Failed to parse ContentType from header `Content-Type",
                )
            })?,
        );
        output = output.set_e_tag(
            crate::protocol_serde::shape_describe_input_device_thumbnail_output::de_e_tag_header(_response_headers).map_err(|_| {
                crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled(
                    "Failed to parse ETag from header `ETag",
                )
            })?,
        );
        output = output.set_last_modified(
            crate::protocol_serde::shape_describe_input_device_thumbnail_output::de_last_modified_header(_response_headers).map_err(|_| {
                crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled(
                    "Failed to parse LastModified from header `Last-Modified",
                )
            })?,
        );
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_input_device_thumbnail_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailOutput,
    crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "BadGatewayException" => crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::BadGatewayException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::BadGatewayExceptionBuilder::default();
                output = crate::protocol_serde::shape_bad_gateway_exception::de_bad_gateway_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "BadRequestException" => crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::BadRequestException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::BadRequestExceptionBuilder::default();
                output = crate::protocol_serde::shape_bad_request_exception::de_bad_request_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ForbiddenException" => crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::ForbiddenException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ForbiddenExceptionBuilder::default();
                output = crate::protocol_serde::shape_forbidden_exception::de_forbidden_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "GatewayTimeoutException" => crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::GatewayTimeoutException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::GatewayTimeoutExceptionBuilder::default();
                output = crate::protocol_serde::shape_gateway_timeout_exception::de_gateway_timeout_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalServerErrorException" => {
            crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::InternalServerErrorException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InternalServerErrorExceptionBuilder::default();
                    output = crate::protocol_serde::shape_internal_server_error_exception::de_internal_server_error_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "NotFoundException" => crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::NotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::NotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_not_found_exception::de_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TooManyRequestsException" => {
            crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::TooManyRequestsException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::TooManyRequestsExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_too_many_requests_exception::de_too_many_requests_exception_json_err(_response_body, output)
                            .map_err(crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        _ => crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailError::generic(generic),
    })
}

pub fn ser_describe_input_device_thumbnail_headers(
    input: &crate::operation::describe_input_device_thumbnail::DescribeInputDeviceThumbnailInput,
    mut builder: ::http::request::Builder,
) -> std::result::Result<::http::request::Builder, ::aws_smithy_types::error::operation::BuildError> {
    if let ::std::option::Option::Some(inner_1) = &input.accept {
        let formatted_2 = inner_1.as_str();
        let header_value = formatted_2;
        let header_value: ::http::HeaderValue = header_value.parse().map_err(|err| {
            ::aws_smithy_types::error::operation::BuildError::invalid_field(
                "accept",
                format!("`{}` cannot be used as a header value: {}", &header_value, err),
            )
        })?;
        builder = builder.header("accept", header_value);
    }
    Ok(builder)
}

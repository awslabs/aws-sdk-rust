// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_input_device_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::describe_input_device::DescribeInputDeviceOutput,
    crate::operation::describe_input_device::DescribeInputDeviceError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::describe_input_device::DescribeInputDeviceError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::describe_input_device::DescribeInputDeviceError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "BadGatewayException" => crate::operation::describe_input_device::DescribeInputDeviceError::BadGatewayException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::BadGatewayExceptionBuilder::default();
                output = crate::protocol_serde::shape_bad_gateway_exception::de_bad_gateway_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_input_device::DescribeInputDeviceError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "BadRequestException" => crate::operation::describe_input_device::DescribeInputDeviceError::BadRequestException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::BadRequestExceptionBuilder::default();
                output = crate::protocol_serde::shape_bad_request_exception::de_bad_request_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_input_device::DescribeInputDeviceError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ForbiddenException" => crate::operation::describe_input_device::DescribeInputDeviceError::ForbiddenException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ForbiddenExceptionBuilder::default();
                output = crate::protocol_serde::shape_forbidden_exception::de_forbidden_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_input_device::DescribeInputDeviceError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "GatewayTimeoutException" => crate::operation::describe_input_device::DescribeInputDeviceError::GatewayTimeoutException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::GatewayTimeoutExceptionBuilder::default();
                output = crate::protocol_serde::shape_gateway_timeout_exception::de_gateway_timeout_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_input_device::DescribeInputDeviceError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InternalServerErrorException" => crate::operation::describe_input_device::DescribeInputDeviceError::InternalServerErrorException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InternalServerErrorExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_internal_server_error_exception::de_internal_server_error_exception_json_err(_response_body, output)
                        .map_err(crate::operation::describe_input_device::DescribeInputDeviceError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "NotFoundException" => crate::operation::describe_input_device::DescribeInputDeviceError::NotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::NotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_not_found_exception::de_not_found_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_input_device::DescribeInputDeviceError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TooManyRequestsException" => crate::operation::describe_input_device::DescribeInputDeviceError::TooManyRequestsException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::TooManyRequestsExceptionBuilder::default();
                output = crate::protocol_serde::shape_too_many_requests_exception::de_too_many_requests_exception_json_err(_response_body, output)
                    .map_err(crate::operation::describe_input_device::DescribeInputDeviceError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::describe_input_device::DescribeInputDeviceError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_describe_input_device_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::describe_input_device::DescribeInputDeviceOutput,
    crate::operation::describe_input_device::DescribeInputDeviceError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::describe_input_device::builders::DescribeInputDeviceOutputBuilder::default();
        output = crate::protocol_serde::shape_describe_input_device::de_describe_input_device(_response_body, output)
            .map_err(crate::operation::describe_input_device::DescribeInputDeviceError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub(crate) fn de_describe_input_device(
    value: &[u8],
    mut builder: crate::operation::describe_input_device::builders::DescribeInputDeviceOutputBuilder,
) -> ::std::result::Result<
    crate::operation::describe_input_device::builders::DescribeInputDeviceOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "arn" => {
                    builder = builder.set_arn(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "availabilityZone" => {
                    builder = builder.set_availability_zone(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "connectionState" => {
                    builder = builder.set_connection_state(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| crate::types::InputDeviceConnectionState::from(u.as_ref())))
                            .transpose()?,
                    );
                }
                "deviceSettingsSyncState" => {
                    builder = builder.set_device_settings_sync_state(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| crate::types::DeviceSettingsSyncState::from(u.as_ref())))
                            .transpose()?,
                    );
                }
                "deviceUpdateStatus" => {
                    builder = builder.set_device_update_status(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| crate::types::DeviceUpdateStatus::from(u.as_ref())))
                            .transpose()?,
                    );
                }
                "hdDeviceSettings" => {
                    builder = builder.set_hd_device_settings(crate::protocol_serde::shape_input_device_hd_settings::de_input_device_hd_settings(
                        tokens,
                    )?);
                }
                "id" => {
                    builder = builder.set_id(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "macAddress" => {
                    builder = builder.set_mac_address(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "medialiveInputArns" => {
                    builder = builder.set_medialive_input_arns(crate::protocol_serde::shape_list_of_string::de_list_of_string(tokens)?);
                }
                "name" => {
                    builder = builder.set_name(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "networkSettings" => {
                    builder = builder
                        .set_network_settings(crate::protocol_serde::shape_input_device_network_settings::de_input_device_network_settings(tokens)?);
                }
                "outputType" => {
                    builder = builder.set_output_type(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| crate::types::InputDeviceOutputType::from(u.as_ref())))
                            .transpose()?,
                    );
                }
                "serialNumber" => {
                    builder = builder.set_serial_number(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                            .transpose()?,
                    );
                }
                "tags" => {
                    builder = builder.set_tags(crate::protocol_serde::shape_tags::de_tags(tokens)?);
                }
                "type" => {
                    builder = builder.set_type(
                        ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                            .map(|s| s.to_unescaped().map(|u| crate::types::InputDeviceType::from(u.as_ref())))
                            .transpose()?,
                    );
                }
                "uhdDeviceSettings" => {
                    builder = builder.set_uhd_device_settings(crate::protocol_serde::shape_input_device_uhd_settings::de_input_device_uhd_settings(
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

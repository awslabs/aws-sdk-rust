// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_put_delivery_channel_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::put_delivery_channel::PutDeliveryChannelOutput,
    crate::operation::put_delivery_channel::PutDeliveryChannelError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::put_delivery_channel::PutDeliveryChannelError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::put_delivery_channel::PutDeliveryChannelError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "InsufficientDeliveryPolicyException" => {
            crate::operation::put_delivery_channel::PutDeliveryChannelError::InsufficientDeliveryPolicyException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InsufficientDeliveryPolicyExceptionBuilder::default();
                    output = crate::protocol_serde::shape_insufficient_delivery_policy_exception::de_insufficient_delivery_policy_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::put_delivery_channel::PutDeliveryChannelError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidDeliveryChannelNameException" => {
            crate::operation::put_delivery_channel::PutDeliveryChannelError::InvalidDeliveryChannelNameException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidDeliveryChannelNameExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_invalid_delivery_channel_name_exception::de_invalid_delivery_channel_name_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::put_delivery_channel::PutDeliveryChannelError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidS3KeyPrefixException" => crate::operation::put_delivery_channel::PutDeliveryChannelError::InvalidS3KeyPrefixException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidS3KeyPrefixExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_invalid_s3_key_prefix_exception::de_invalid_s3_key_prefix_exception_json_err(_response_body, output)
                        .map_err(crate::operation::put_delivery_channel::PutDeliveryChannelError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidS3KmsKeyArnException" => crate::operation::put_delivery_channel::PutDeliveryChannelError::InvalidS3KmsKeyArnException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidS3KmsKeyArnExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_s3_kms_key_arn_exception::de_invalid_s3_kms_key_arn_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::put_delivery_channel::PutDeliveryChannelError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidSNSTopicARNException" => crate::operation::put_delivery_channel::PutDeliveryChannelError::InvalidSnsTopicArnException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidSnsTopicArnExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_invalid_sns_topic_arn_exception::de_invalid_sns_topic_arn_exception_json_err(_response_body, output)
                        .map_err(crate::operation::put_delivery_channel::PutDeliveryChannelError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "MaxNumberOfDeliveryChannelsExceededException" => {
            crate::operation::put_delivery_channel::PutDeliveryChannelError::MaxNumberOfDeliveryChannelsExceededException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::MaxNumberOfDeliveryChannelsExceededExceptionBuilder::default();
                    output = crate::protocol_serde::shape_max_number_of_delivery_channels_exceeded_exception::de_max_number_of_delivery_channels_exceeded_exception_json_err(_response_body, output).map_err(crate::operation::put_delivery_channel::PutDeliveryChannelError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "NoAvailableConfigurationRecorderException" => {
            crate::operation::put_delivery_channel::PutDeliveryChannelError::NoAvailableConfigurationRecorderException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::NoAvailableConfigurationRecorderExceptionBuilder::default();
                    output = crate::protocol_serde::shape_no_available_configuration_recorder_exception::de_no_available_configuration_recorder_exception_json_err(_response_body, output).map_err(crate::operation::put_delivery_channel::PutDeliveryChannelError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "NoSuchBucketException" => crate::operation::put_delivery_channel::PutDeliveryChannelError::NoSuchBucketException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::NoSuchBucketExceptionBuilder::default();
                output = crate::protocol_serde::shape_no_such_bucket_exception::de_no_such_bucket_exception_json_err(_response_body, output)
                    .map_err(crate::operation::put_delivery_channel::PutDeliveryChannelError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::put_delivery_channel::PutDeliveryChannelError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_put_delivery_channel_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::put_delivery_channel::PutDeliveryChannelOutput,
    crate::operation::put_delivery_channel::PutDeliveryChannelError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::put_delivery_channel::builders::PutDeliveryChannelOutputBuilder::default();
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_put_delivery_channel_input(
    input: &crate::operation::put_delivery_channel::PutDeliveryChannelInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_put_delivery_channel_input::ser_put_delivery_channel_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

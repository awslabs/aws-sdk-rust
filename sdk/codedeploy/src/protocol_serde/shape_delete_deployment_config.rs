// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_delete_deployment_config_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::delete_deployment_config::DeleteDeploymentConfigOutput,
    crate::operation::delete_deployment_config::DeleteDeploymentConfigError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::delete_deployment_config::DeleteDeploymentConfigError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => {
            return Err(crate::operation::delete_deployment_config::DeleteDeploymentConfigError::unhandled(
                generic,
            ))
        }
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "DeploymentConfigInUseException" => {
            crate::operation::delete_deployment_config::DeleteDeploymentConfigError::DeploymentConfigInUseException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DeploymentConfigInUseExceptionBuilder::default();
                    output = crate::protocol_serde::shape_deployment_config_in_use_exception::de_deployment_config_in_use_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::delete_deployment_config::DeleteDeploymentConfigError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "DeploymentConfigNameRequiredException" => {
            crate::operation::delete_deployment_config::DeleteDeploymentConfigError::DeploymentConfigNameRequiredException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DeploymentConfigNameRequiredExceptionBuilder::default();
                    output = crate::protocol_serde::shape_deployment_config_name_required_exception::de_deployment_config_name_required_exception_json_err(_response_body, output).map_err(crate::operation::delete_deployment_config::DeleteDeploymentConfigError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidDeploymentConfigNameException" => {
            crate::operation::delete_deployment_config::DeleteDeploymentConfigError::InvalidDeploymentConfigNameException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidDeploymentConfigNameExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_invalid_deployment_config_name_exception::de_invalid_deployment_config_name_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::delete_deployment_config::DeleteDeploymentConfigError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidOperationException" => crate::operation::delete_deployment_config::DeleteDeploymentConfigError::InvalidOperationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidOperationExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_operation_exception::de_invalid_operation_exception_json_err(_response_body, output)
                    .map_err(crate::operation::delete_deployment_config::DeleteDeploymentConfigError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::delete_deployment_config::DeleteDeploymentConfigError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_delete_deployment_config_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::delete_deployment_config::DeleteDeploymentConfigOutput,
    crate::operation::delete_deployment_config::DeleteDeploymentConfigError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::delete_deployment_config::builders::DeleteDeploymentConfigOutputBuilder::default();
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_delete_deployment_config_input(
    input: &crate::operation::delete_deployment_config::DeleteDeploymentConfigInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_delete_deployment_config_input::ser_delete_deployment_config_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

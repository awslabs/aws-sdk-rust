// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_update_deployment_group_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::update_deployment_group::UpdateDeploymentGroupOutput,
    crate::operation::update_deployment_group::UpdateDeploymentGroupError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "AlarmsLimitExceededException" => crate::operation::update_deployment_group::UpdateDeploymentGroupError::AlarmsLimitExceededException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::AlarmsLimitExceededExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_alarms_limit_exceeded_exception::de_alarms_limit_exceeded_exception_json_err(_response_body, output)
                        .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ApplicationDoesNotExistException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::ApplicationDoesNotExistException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ApplicationDoesNotExistExceptionBuilder::default();
                    output = crate::protocol_serde::shape_application_does_not_exist_exception::de_application_does_not_exist_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "ApplicationNameRequiredException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::ApplicationNameRequiredException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ApplicationNameRequiredExceptionBuilder::default();
                    output = crate::protocol_serde::shape_application_name_required_exception::de_application_name_required_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "DeploymentConfigDoesNotExistException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::DeploymentConfigDoesNotExistException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DeploymentConfigDoesNotExistExceptionBuilder::default();
                    output = crate::protocol_serde::shape_deployment_config_does_not_exist_exception::de_deployment_config_does_not_exist_exception_json_err(_response_body, output).map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "DeploymentGroupAlreadyExistsException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::DeploymentGroupAlreadyExistsException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DeploymentGroupAlreadyExistsExceptionBuilder::default();
                    output = crate::protocol_serde::shape_deployment_group_already_exists_exception::de_deployment_group_already_exists_exception_json_err(_response_body, output).map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "DeploymentGroupDoesNotExistException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::DeploymentGroupDoesNotExistException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DeploymentGroupDoesNotExistExceptionBuilder::default();
                    output = crate::protocol_serde::shape_deployment_group_does_not_exist_exception::de_deployment_group_does_not_exist_exception_json_err(_response_body, output).map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "DeploymentGroupNameRequiredException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::DeploymentGroupNameRequiredException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DeploymentGroupNameRequiredExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_deployment_group_name_required_exception::de_deployment_group_name_required_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "ECSServiceMappingLimitExceededException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::EcsServiceMappingLimitExceededException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::EcsServiceMappingLimitExceededExceptionBuilder::default();
                    output = crate::protocol_serde::shape_ecs_service_mapping_limit_exceeded_exception::de_ecs_service_mapping_limit_exceeded_exception_json_err(_response_body, output).map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidAlarmConfigException" => crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidAlarmConfigException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidAlarmConfigExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_invalid_alarm_config_exception::de_invalid_alarm_config_exception_json_err(_response_body, output)
                        .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidApplicationNameException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidApplicationNameException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidApplicationNameExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_application_name_exception::de_invalid_application_name_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidAutoRollbackConfigException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidAutoRollbackConfigException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidAutoRollbackConfigExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_auto_rollback_config_exception::de_invalid_auto_rollback_config_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidAutoScalingGroupException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidAutoScalingGroupException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidAutoScalingGroupExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_auto_scaling_group_exception::de_invalid_auto_scaling_group_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidBlueGreenDeploymentConfigurationException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidBlueGreenDeploymentConfigurationException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidBlueGreenDeploymentConfigurationExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_blue_green_deployment_configuration_exception::de_invalid_blue_green_deployment_configuration_exception_json_err(_response_body, output).map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
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
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidDeploymentConfigNameException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidDeploymentConfigNameExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_invalid_deployment_config_name_exception::de_invalid_deployment_config_name_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidDeploymentGroupNameException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidDeploymentGroupNameException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidDeploymentGroupNameExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_invalid_deployment_group_name_exception::de_invalid_deployment_group_name_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidDeploymentStyleException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidDeploymentStyleException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidDeploymentStyleExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_deployment_style_exception::de_invalid_deployment_style_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidEC2TagCombinationException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidEc2TagCombinationException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidEc2TagCombinationExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_ec2_tag_combination_exception::de_invalid_ec2_tag_combination_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidEC2TagException" => crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidEc2TagException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidEc2TagExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_ec2_tag_exception::de_invalid_ec2_tag_exception_json_err(_response_body, output)
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidECSServiceException" => crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidEcsServiceException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidEcsServiceExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_invalid_ecs_service_exception::de_invalid_ecs_service_exception_json_err(_response_body, output)
                        .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidInputException" => crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidInputException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidInputExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_input_exception::de_invalid_input_exception_json_err(_response_body, output)
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidLoadBalancerInfoException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidLoadBalancerInfoException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidLoadBalancerInfoExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_load_balancer_info_exception::de_invalid_load_balancer_info_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidOnPremisesTagCombinationException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidOnPremisesTagCombinationException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidOnPremisesTagCombinationExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_on_premises_tag_combination_exception::de_invalid_on_premises_tag_combination_exception_json_err(_response_body, output).map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidRoleException" => crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidRoleException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidRoleExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_role_exception::de_invalid_role_exception_json_err(_response_body, output)
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidTagException" => crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidTagException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidTagExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_tag_exception::de_invalid_tag_exception_json_err(_response_body, output)
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidTargetGroupPairException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidTargetGroupPairException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidTargetGroupPairExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_target_group_pair_exception::de_invalid_target_group_pair_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidTrafficRoutingConfigurationException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidTrafficRoutingConfigurationException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidTrafficRoutingConfigurationExceptionBuilder::default();
                    output = crate::protocol_serde::shape_invalid_traffic_routing_configuration_exception::de_invalid_traffic_routing_configuration_exception_json_err(_response_body, output).map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidTriggerConfigException" => crate::operation::update_deployment_group::UpdateDeploymentGroupError::InvalidTriggerConfigException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidTriggerConfigExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_trigger_config_exception::de_invalid_trigger_config_exception_json_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "LifecycleHookLimitExceededException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::LifecycleHookLimitExceededException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::LifecycleHookLimitExceededExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_lifecycle_hook_limit_exceeded_exception::de_lifecycle_hook_limit_exceeded_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "TagSetListLimitExceededException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::TagSetListLimitExceededException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::TagSetListLimitExceededExceptionBuilder::default();
                    output = crate::protocol_serde::shape_tag_set_list_limit_exceeded_exception::de_tag_set_list_limit_exceeded_exception_json_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "ThrottlingException" => crate::operation::update_deployment_group::UpdateDeploymentGroupError::ThrottlingException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ThrottlingExceptionBuilder::default();
                output = crate::protocol_serde::shape_throttling_exception::de_throttling_exception_json_err(_response_body, output)
                    .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TriggerTargetsLimitExceededException" => {
            crate::operation::update_deployment_group::UpdateDeploymentGroupError::TriggerTargetsLimitExceededException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::TriggerTargetsLimitExceededExceptionBuilder::default();
                    output =
                        crate::protocol_serde::shape_trigger_targets_limit_exceeded_exception::de_trigger_targets_limit_exceeded_exception_json_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        _ => crate::operation::update_deployment_group::UpdateDeploymentGroupError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_update_deployment_group_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::update_deployment_group::UpdateDeploymentGroupOutput,
    crate::operation::update_deployment_group::UpdateDeploymentGroupError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::update_deployment_group::builders::UpdateDeploymentGroupOutputBuilder::default();
        output = crate::protocol_serde::shape_update_deployment_group::de_update_deployment_group(_response_body, output)
            .map_err(crate::operation::update_deployment_group::UpdateDeploymentGroupError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

pub fn ser_update_deployment_group_input(
    input: &crate::operation::update_deployment_group::UpdateDeploymentGroupInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    let mut object = ::aws_smithy_json::serialize::JsonObjectWriter::new(&mut out);
    crate::protocol_serde::shape_update_deployment_group_input::ser_update_deployment_group_input_input(&mut object, input)?;
    object.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

pub(crate) fn de_update_deployment_group(
    value: &[u8],
    mut builder: crate::operation::update_deployment_group::builders::UpdateDeploymentGroupOutputBuilder,
) -> ::std::result::Result<
    crate::operation::update_deployment_group::builders::UpdateDeploymentGroupOutputBuilder,
    ::aws_smithy_json::deserialize::error::DeserializeError,
> {
    let mut tokens_owned = ::aws_smithy_json::deserialize::json_token_iter(crate::protocol_serde::or_empty_doc(value)).peekable();
    let tokens = &mut tokens_owned;
    ::aws_smithy_json::deserialize::token::expect_start_object(tokens.next())?;
    loop {
        match tokens.next().transpose()? {
            Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
            Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                "hooksNotCleanedUp" => {
                    builder =
                        builder.set_hooks_not_cleaned_up(crate::protocol_serde::shape_auto_scaling_group_list::de_auto_scaling_group_list(tokens)?);
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

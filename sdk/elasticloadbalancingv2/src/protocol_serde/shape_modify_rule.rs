// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_modify_rule_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::modify_rule::ModifyRuleOutput, crate::operation::modify_rule::ModifyRuleError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::modify_rule::ModifyRuleError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "IncompatibleProtocols" => crate::operation::modify_rule::ModifyRuleError::IncompatibleProtocolsException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::IncompatibleProtocolsExceptionBuilder::default();
                output = crate::protocol_serde::shape_incompatible_protocols_exception::de_incompatible_protocols_exception_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidLoadBalancerAction" => crate::operation::modify_rule::ModifyRuleError::InvalidLoadBalancerActionException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidLoadBalancerActionExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_load_balancer_action_exception::de_invalid_load_balancer_action_exception_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "OperationNotPermitted" => crate::operation::modify_rule::ModifyRuleError::OperationNotPermittedException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::OperationNotPermittedExceptionBuilder::default();
                output = crate::protocol_serde::shape_operation_not_permitted_exception::de_operation_not_permitted_exception_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "RuleNotFound" => crate::operation::modify_rule::ModifyRuleError::RuleNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::RuleNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_rule_not_found_exception::de_rule_not_found_exception_xml_err(_response_body, output)
                    .map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TargetGroupAssociationLimit" => crate::operation::modify_rule::ModifyRuleError::TargetGroupAssociationLimitException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::TargetGroupAssociationLimitExceptionBuilder::default();
                output = crate::protocol_serde::shape_target_group_association_limit_exception::de_target_group_association_limit_exception_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TargetGroupNotFound" => crate::operation::modify_rule::ModifyRuleError::TargetGroupNotFoundException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::TargetGroupNotFoundExceptionBuilder::default();
                output = crate::protocol_serde::shape_target_group_not_found_exception::de_target_group_not_found_exception_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TooManyActions" => crate::operation::modify_rule::ModifyRuleError::TooManyActionsException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::TooManyActionsExceptionBuilder::default();
                output = crate::protocol_serde::shape_too_many_actions_exception::de_too_many_actions_exception_xml_err(_response_body, output)
                    .map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TooManyRegistrationsForTargetId" => crate::operation::modify_rule::ModifyRuleError::TooManyRegistrationsForTargetIdException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::TooManyRegistrationsForTargetIdExceptionBuilder::default();
                output = crate::protocol_serde::shape_too_many_registrations_for_target_id_exception::de_too_many_registrations_for_target_id_exception_xml_err(_response_body, output).map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TooManyTargets" => crate::operation::modify_rule::ModifyRuleError::TooManyTargetsException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::TooManyTargetsExceptionBuilder::default();
                output = crate::protocol_serde::shape_too_many_targets_exception::de_too_many_targets_exception_xml_err(_response_body, output)
                    .map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "TooManyUniqueTargetGroupsPerLoadBalancer" => {
            crate::operation::modify_rule::ModifyRuleError::TooManyUniqueTargetGroupsPerLoadBalancerException({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::TooManyUniqueTargetGroupsPerLoadBalancerExceptionBuilder::default();
                    output = crate::protocol_serde::shape_too_many_unique_target_groups_per_load_balancer_exception::de_too_many_unique_target_groups_per_load_balancer_exception_xml_err(_response_body, output).map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "UnsupportedProtocol" => crate::operation::modify_rule::ModifyRuleError::UnsupportedProtocolException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::UnsupportedProtocolExceptionBuilder::default();
                output =
                    crate::protocol_serde::shape_unsupported_protocol_exception::de_unsupported_protocol_exception_xml_err(_response_body, output)
                        .map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::modify_rule::ModifyRuleError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_modify_rule_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::modify_rule::ModifyRuleOutput, crate::operation::modify_rule::ModifyRuleError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::modify_rule::builders::ModifyRuleOutputBuilder::default();
        output = crate::protocol_serde::shape_modify_rule::de_modify_rule(_response_body, output)
            .map_err(crate::operation::modify_rule::ModifyRuleError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_modify_rule(
    inp: &[u8],
    mut builder: crate::operation::modify_rule::builders::ModifyRuleOutputBuilder,
) -> std::result::Result<crate::operation::modify_rule::builders::ModifyRuleOutputBuilder, ::aws_smithy_xml::decode::XmlDecodeError> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("ModifyRuleResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected ModifyRuleResponse got {:?}",
            start_el
        )));
    }
    if let Some(mut result_tag) = decoder.next_tag() {
        let start_el = result_tag.start_el();
        if !(start_el.matches("ModifyRuleResult")) {
            return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
                "invalid result, expected ModifyRuleResult got {:?}",
                start_el
            )));
        }
        while let Some(mut tag) = result_tag.next_tag() {
            match tag.start_el() {
            s if s.matches("Rules") /* Rules com.amazonaws.elasticloadbalancingv2.synthetic#ModifyRuleOutput$Rules */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_rules::de_rules(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_rules(var_1);
            }
            ,
            _ => {}
        }
        }
    } else {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom("expected ModifyRuleResult tag"));
    };
    Ok(builder)
}

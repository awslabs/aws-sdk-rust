// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_delete_user_group_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::delete_user_group::DeleteUserGroupOutput, crate::operation::delete_user_group::DeleteUserGroupError> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::delete_user_group::DeleteUserGroupError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::delete_user_group::DeleteUserGroupError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "InvalidParameterValue" => crate::operation::delete_user_group::DeleteUserGroupError::InvalidParameterValueException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidParameterValueExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_parameter_value_exception::de_invalid_parameter_value_exception_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::delete_user_group::DeleteUserGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidUserGroupState" => crate::operation::delete_user_group::DeleteUserGroupError::InvalidUserGroupStateFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidUserGroupStateFaultBuilder::default();
                output =
                    crate::protocol_serde::shape_invalid_user_group_state_fault::de_invalid_user_group_state_fault_xml_err(_response_body, output)
                        .map_err(crate::operation::delete_user_group::DeleteUserGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ServiceLinkedRoleNotFoundFault" => crate::operation::delete_user_group::DeleteUserGroupError::ServiceLinkedRoleNotFoundFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ServiceLinkedRoleNotFoundFaultBuilder::default();
                output = crate::protocol_serde::shape_service_linked_role_not_found_fault::de_service_linked_role_not_found_fault_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::delete_user_group::DeleteUserGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "UserGroupNotFound" => crate::operation::delete_user_group::DeleteUserGroupError::UserGroupNotFoundFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::UserGroupNotFoundFaultBuilder::default();
                output = crate::protocol_serde::shape_user_group_not_found_fault::de_user_group_not_found_fault_xml_err(_response_body, output)
                    .map_err(crate::operation::delete_user_group::DeleteUserGroupError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::delete_user_group::DeleteUserGroupError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_delete_user_group_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<crate::operation::delete_user_group::DeleteUserGroupOutput, crate::operation::delete_user_group::DeleteUserGroupError> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::delete_user_group::builders::DeleteUserGroupOutputBuilder::default();
        output = crate::protocol_serde::shape_delete_user_group::de_delete_user_group(_response_body, output)
            .map_err(crate::operation::delete_user_group::DeleteUserGroupError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_delete_user_group(
    inp: &[u8],
    mut builder: crate::operation::delete_user_group::builders::DeleteUserGroupOutputBuilder,
) -> std::result::Result<crate::operation::delete_user_group::builders::DeleteUserGroupOutputBuilder, ::aws_smithy_xml::decode::XmlDecodeError> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("DeleteUserGroupResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected DeleteUserGroupResponse got {:?}",
            start_el
        )));
    }
    if let Some(mut result_tag) = decoder.next_tag() {
        let start_el = result_tag.start_el();
        if !(start_el.matches("DeleteUserGroupResult")) {
            return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
                "invalid result, expected DeleteUserGroupResult got {:?}",
                start_el
            )));
        }
        while let Some(mut tag) = result_tag.next_tag() {
            match tag.start_el() {
            s if s.matches("UserGroupId") /* UserGroupId com.amazonaws.elasticache.synthetic#DeleteUserGroupOutput$UserGroupId */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_user_group_id(var_1);
            }
            ,
            s if s.matches("Status") /* Status com.amazonaws.elasticache.synthetic#DeleteUserGroupOutput$Status */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_status(var_2);
            }
            ,
            s if s.matches("Engine") /* Engine com.amazonaws.elasticache.synthetic#DeleteUserGroupOutput$Engine */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_engine(var_3);
            }
            ,
            s if s.matches("UserIds") /* UserIds com.amazonaws.elasticache.synthetic#DeleteUserGroupOutput$UserIds */ =>  {
                let var_4 =
                    Some(
                        crate::protocol_serde::shape_user_id_list::de_user_id_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_user_ids(var_4);
            }
            ,
            s if s.matches("MinimumEngineVersion") /* MinimumEngineVersion com.amazonaws.elasticache.synthetic#DeleteUserGroupOutput$MinimumEngineVersion */ =>  {
                let var_5 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_minimum_engine_version(var_5);
            }
            ,
            s if s.matches("PendingChanges") /* PendingChanges com.amazonaws.elasticache.synthetic#DeleteUserGroupOutput$PendingChanges */ =>  {
                let var_6 =
                    Some(
                        crate::protocol_serde::shape_user_group_pending_changes::de_user_group_pending_changes(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_pending_changes(var_6);
            }
            ,
            s if s.matches("ReplicationGroups") /* ReplicationGroups com.amazonaws.elasticache.synthetic#DeleteUserGroupOutput$ReplicationGroups */ =>  {
                let var_7 =
                    Some(
                        crate::protocol_serde::shape_ug_replication_group_id_list::de_ug_replication_group_id_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_replication_groups(var_7);
            }
            ,
            s if s.matches("ServerlessCaches") /* ServerlessCaches com.amazonaws.elasticache.synthetic#DeleteUserGroupOutput$ServerlessCaches */ =>  {
                let var_8 =
                    Some(
                        crate::protocol_serde::shape_ug_serverless_cache_id_list::de_ug_serverless_cache_id_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_serverless_caches(var_8);
            }
            ,
            s if s.matches("ARN") /* ARN com.amazonaws.elasticache.synthetic#DeleteUserGroupOutput$ARN */ =>  {
                let var_9 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_arn(var_9);
            }
            ,
            _ => {}
        }
        }
    } else {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom("expected DeleteUserGroupResult tag"));
    };
    Ok(builder)
}

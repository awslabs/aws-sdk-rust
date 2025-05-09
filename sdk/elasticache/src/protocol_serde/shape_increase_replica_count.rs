// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_increase_replica_count_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::increase_replica_count::IncreaseReplicaCountOutput,
    crate::operation::increase_replica_count::IncreaseReplicaCountError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "ClusterQuotaForCustomerExceeded" => {
            crate::operation::increase_replica_count::IncreaseReplicaCountError::ClusterQuotaForCustomerExceededFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::ClusterQuotaForCustomerExceededFaultBuilder::default();
                    output =
                        crate::protocol_serde::shape_cluster_quota_for_customer_exceeded_fault::de_cluster_quota_for_customer_exceeded_fault_xml_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InsufficientCacheClusterCapacity" => {
            crate::operation::increase_replica_count::IncreaseReplicaCountError::InsufficientCacheClusterCapacityFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InsufficientCacheClusterCapacityFaultBuilder::default();
                    output =
                        crate::protocol_serde::shape_insufficient_cache_cluster_capacity_fault::de_insufficient_cache_cluster_capacity_fault_xml_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidCacheClusterState" => crate::operation::increase_replica_count::IncreaseReplicaCountError::InvalidCacheClusterStateFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidCacheClusterStateFaultBuilder::default();
                output = crate::protocol_serde::shape_invalid_cache_cluster_state_fault::de_invalid_cache_cluster_state_fault_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidKMSKeyFault" => crate::operation::increase_replica_count::IncreaseReplicaCountError::InvalidKmsKeyFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidKmsKeyFaultBuilder::default();
                output = crate::protocol_serde::shape_invalid_kms_key_fault::de_invalid_kms_key_fault_xml_err(_response_body, output)
                    .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidParameterCombination" => crate::operation::increase_replica_count::IncreaseReplicaCountError::InvalidParameterCombinationException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidParameterCombinationExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_parameter_combination_exception::de_invalid_parameter_combination_exception_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidParameterValue" => crate::operation::increase_replica_count::IncreaseReplicaCountError::InvalidParameterValueException({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidParameterValueExceptionBuilder::default();
                output = crate::protocol_serde::shape_invalid_parameter_value_exception::de_invalid_parameter_value_exception_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidReplicationGroupState" => crate::operation::increase_replica_count::IncreaseReplicaCountError::InvalidReplicationGroupStateFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidReplicationGroupStateFaultBuilder::default();
                output = crate::protocol_serde::shape_invalid_replication_group_state_fault::de_invalid_replication_group_state_fault_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidVPCNetworkStateFault" => crate::operation::increase_replica_count::IncreaseReplicaCountError::InvalidVpcNetworkStateFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidVpcNetworkStateFaultBuilder::default();
                output =
                    crate::protocol_serde::shape_invalid_vpc_network_state_fault::de_invalid_vpc_network_state_fault_xml_err(_response_body, output)
                        .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "NodeGroupsPerReplicationGroupQuotaExceeded" => {
            crate::operation::increase_replica_count::IncreaseReplicaCountError::NodeGroupsPerReplicationGroupQuotaExceededFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::NodeGroupsPerReplicationGroupQuotaExceededFaultBuilder::default();
                    output = crate::protocol_serde::shape_node_groups_per_replication_group_quota_exceeded_fault::de_node_groups_per_replication_group_quota_exceeded_fault_xml_err(_response_body, output).map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "NodeQuotaForCustomerExceeded" => crate::operation::increase_replica_count::IncreaseReplicaCountError::NodeQuotaForCustomerExceededFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::NodeQuotaForCustomerExceededFaultBuilder::default();
                output = crate::protocol_serde::shape_node_quota_for_customer_exceeded_fault::de_node_quota_for_customer_exceeded_fault_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "NoOperationFault" => crate::operation::increase_replica_count::IncreaseReplicaCountError::NoOperationFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::NoOperationFaultBuilder::default();
                output = crate::protocol_serde::shape_no_operation_fault::de_no_operation_fault_xml_err(_response_body, output)
                    .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ReplicationGroupNotFoundFault" => crate::operation::increase_replica_count::IncreaseReplicaCountError::ReplicationGroupNotFoundFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ReplicationGroupNotFoundFaultBuilder::default();
                output = crate::protocol_serde::shape_replication_group_not_found_fault::de_replication_group_not_found_fault_xml_err(
                    _response_body,
                    output,
                )
                .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::increase_replica_count::IncreaseReplicaCountError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_increase_replica_count_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::increase_replica_count::IncreaseReplicaCountOutput,
    crate::operation::increase_replica_count::IncreaseReplicaCountError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::increase_replica_count::builders::IncreaseReplicaCountOutputBuilder::default();
        output = crate::protocol_serde::shape_increase_replica_count::de_increase_replica_count(_response_body, output)
            .map_err(crate::operation::increase_replica_count::IncreaseReplicaCountError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_increase_replica_count(
    inp: &[u8],
    mut builder: crate::operation::increase_replica_count::builders::IncreaseReplicaCountOutputBuilder,
) -> std::result::Result<
    crate::operation::increase_replica_count::builders::IncreaseReplicaCountOutputBuilder,
    ::aws_smithy_xml::decode::XmlDecodeError,
> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("IncreaseReplicaCountResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected IncreaseReplicaCountResponse got {:?}",
            start_el
        )));
    }
    if let Some(mut result_tag) = decoder.next_tag() {
        let start_el = result_tag.start_el();
        if !(start_el.matches("IncreaseReplicaCountResult")) {
            return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
                "invalid result, expected IncreaseReplicaCountResult got {:?}",
                start_el
            )));
        }
        while let Some(mut tag) = result_tag.next_tag() {
            match tag.start_el() {
            s if s.matches("ReplicationGroup") /* ReplicationGroup com.amazonaws.elasticache.synthetic#IncreaseReplicaCountOutput$ReplicationGroup */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_replication_group::de_replication_group(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_replication_group(var_1);
            }
            ,
            _ => {}
        }
        }
    } else {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(
            "expected IncreaseReplicaCountResult tag",
        ));
    };
    Ok(builder)
}

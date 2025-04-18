// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_restore_db_cluster_to_point_in_time_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::restore_db_cluster_to_point_in_time::RestoreDbClusterToPointInTimeOutput,
    crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "DBClusterAlreadyExistsFault" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::DbClusterAlreadyExistsFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DbClusterAlreadyExistsFaultBuilder::default();
                    output = crate::protocol_serde::shape_db_cluster_already_exists_fault::de_db_cluster_already_exists_fault_xml_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "DBClusterNotFoundFault" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::DbClusterNotFoundFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DbClusterNotFoundFaultBuilder::default();
                    output = crate::protocol_serde::shape_db_cluster_not_found_fault::de_db_cluster_not_found_fault_xml_err(_response_body, output)
                        .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "DBClusterQuotaExceededFault" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::DbClusterQuotaExceededFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DbClusterQuotaExceededFaultBuilder::default();
                    output = crate::protocol_serde::shape_db_cluster_quota_exceeded_fault::de_db_cluster_quota_exceeded_fault_xml_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "DBClusterSnapshotNotFoundFault" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::DbClusterSnapshotNotFoundFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DbClusterSnapshotNotFoundFaultBuilder::default();
                    output = crate::protocol_serde::shape_db_cluster_snapshot_not_found_fault::de_db_cluster_snapshot_not_found_fault_xml_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "DBSubnetGroupNotFoundFault" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::DbSubnetGroupNotFoundFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::DbSubnetGroupNotFoundFaultBuilder::default();
                    output = crate::protocol_serde::shape_db_subnet_group_not_found_fault::de_db_subnet_group_not_found_fault_xml_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InsufficientDBClusterCapacityFault" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::InsufficientDbClusterCapacityFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InsufficientDbClusterCapacityFaultBuilder::default();
                    output = crate::protocol_serde::shape_insufficient_db_cluster_capacity_fault::de_insufficient_db_cluster_capacity_fault_xml_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InsufficientStorageClusterCapacity" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::InsufficientStorageClusterCapacityFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InsufficientStorageClusterCapacityFaultBuilder::default();
                    output = crate::protocol_serde::shape_insufficient_storage_cluster_capacity_fault::de_insufficient_storage_cluster_capacity_fault_xml_err(_response_body, output).map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidDBClusterSnapshotStateFault" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::InvalidDbClusterSnapshotStateFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidDbClusterSnapshotStateFaultBuilder::default();
                    output =
                        crate::protocol_serde::shape_invalid_db_cluster_snapshot_state_fault::de_invalid_db_cluster_snapshot_state_fault_xml_err(
                            _response_body,
                            output,
                        )
                        .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidDBClusterStateFault" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::InvalidDbClusterStateFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidDbClusterStateFaultBuilder::default();
                    output = crate::protocol_serde::shape_invalid_db_cluster_state_fault::de_invalid_db_cluster_state_fault_xml_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidDBSnapshotState" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::InvalidDbSnapshotStateFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidDbSnapshotStateFaultBuilder::default();
                    output = crate::protocol_serde::shape_invalid_db_snapshot_state_fault::de_invalid_db_snapshot_state_fault_xml_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "InvalidRestoreFault" => crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::InvalidRestoreFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidRestoreFaultBuilder::default();
                output = crate::protocol_serde::shape_invalid_restore_fault::de_invalid_restore_fault_xml_err(_response_body, output)
                    .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidSubnet" => crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::InvalidSubnet({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidSubnetBuilder::default();
                output = crate::protocol_serde::shape_invalid_subnet::de_invalid_subnet_xml_err(_response_body, output)
                    .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidVPCNetworkStateFault" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::InvalidVpcNetworkStateFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::InvalidVpcNetworkStateFaultBuilder::default();
                    output = crate::protocol_serde::shape_invalid_vpc_network_state_fault::de_invalid_vpc_network_state_fault_xml_err(
                        _response_body,
                        output,
                    )
                    .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "KMSKeyNotAccessibleFault" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::KmsKeyNotAccessibleFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::KmsKeyNotAccessibleFaultBuilder::default();
                    output =
                        crate::protocol_serde::shape_kms_key_not_accessible_fault::de_kms_key_not_accessible_fault_xml_err(_response_body, output)
                            .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        "StorageQuotaExceeded" => {
            crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::StorageQuotaExceededFault({
                #[allow(unused_mut)]
                let mut tmp = {
                    #[allow(unused_mut)]
                    let mut output = crate::types::error::builders::StorageQuotaExceededFaultBuilder::default();
                    output =
                        crate::protocol_serde::shape_storage_quota_exceeded_fault::de_storage_quota_exceeded_fault_xml_err(_response_body, output)
                            .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
                    let output = output.meta(generic);
                    output.build()
                };
                if tmp.message.is_none() {
                    tmp.message = _error_message;
                }
                tmp
            })
        }
        _ => crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_restore_db_cluster_to_point_in_time_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::restore_db_cluster_to_point_in_time::RestoreDbClusterToPointInTimeOutput,
    crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::restore_db_cluster_to_point_in_time::builders::RestoreDbClusterToPointInTimeOutputBuilder::default();
        output = crate::protocol_serde::shape_restore_db_cluster_to_point_in_time::de_restore_db_cluster_to_point_in_time(_response_body, output)
            .map_err(crate::operation::restore_db_cluster_to_point_in_time::RestoreDBClusterToPointInTimeError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_restore_db_cluster_to_point_in_time(
    inp: &[u8],
    mut builder: crate::operation::restore_db_cluster_to_point_in_time::builders::RestoreDbClusterToPointInTimeOutputBuilder,
) -> std::result::Result<
    crate::operation::restore_db_cluster_to_point_in_time::builders::RestoreDbClusterToPointInTimeOutputBuilder,
    ::aws_smithy_xml::decode::XmlDecodeError,
> {
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("RestoreDBClusterToPointInTimeResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected RestoreDBClusterToPointInTimeResponse got {:?}",
            start_el
        )));
    }
    if let Some(mut result_tag) = decoder.next_tag() {
        let start_el = result_tag.start_el();
        if !(start_el.matches("RestoreDBClusterToPointInTimeResult")) {
            return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
                "invalid result, expected RestoreDBClusterToPointInTimeResult got {:?}",
                start_el
            )));
        }
        while let Some(mut tag) = result_tag.next_tag() {
            match tag.start_el() {
            s if s.matches("DBCluster") /* DBCluster com.amazonaws.docdb.synthetic#RestoreDBClusterToPointInTimeOutput$DBCluster */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_db_cluster::de_db_cluster(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_db_cluster(var_1);
            }
            ,
            _ => {}
        }
        }
    } else {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(
            "expected RestoreDBClusterToPointInTimeResult tag",
        ));
    };
    Ok(builder)
}

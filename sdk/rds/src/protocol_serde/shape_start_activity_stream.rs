// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::unnecessary_wraps)]
pub fn de_start_activity_stream_http_error(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::start_activity_stream::StartActivityStreamOutput,
    crate::operation::start_activity_stream::StartActivityStreamError,
> {
    #[allow(unused_mut)]
    let mut generic_builder = crate::protocol_serde::parse_http_error_metadata(_response_status, _response_headers, _response_body)
        .map_err(crate::operation::start_activity_stream::StartActivityStreamError::unhandled)?;
    generic_builder = ::aws_types::request_id::apply_request_id(generic_builder, _response_headers);
    let generic = generic_builder.build();
    let error_code = match generic.code() {
        Some(code) => code,
        None => return Err(crate::operation::start_activity_stream::StartActivityStreamError::unhandled(generic)),
    };

    let _error_message = generic.message().map(|msg| msg.to_owned());
    Err(match error_code {
        "DBClusterNotFoundFault" => crate::operation::start_activity_stream::StartActivityStreamError::DbClusterNotFoundFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::DbClusterNotFoundFaultBuilder::default();
                output = crate::protocol_serde::shape_db_cluster_not_found_fault::de_db_cluster_not_found_fault_xml_err(_response_body, output)
                    .map_err(crate::operation::start_activity_stream::StartActivityStreamError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "DBInstanceNotFound" => crate::operation::start_activity_stream::StartActivityStreamError::DbInstanceNotFoundFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::DbInstanceNotFoundFaultBuilder::default();
                output = crate::protocol_serde::shape_db_instance_not_found_fault::de_db_instance_not_found_fault_xml_err(_response_body, output)
                    .map_err(crate::operation::start_activity_stream::StartActivityStreamError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidDBClusterStateFault" => crate::operation::start_activity_stream::StartActivityStreamError::InvalidDbClusterStateFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidDbClusterStateFaultBuilder::default();
                output =
                    crate::protocol_serde::shape_invalid_db_cluster_state_fault::de_invalid_db_cluster_state_fault_xml_err(_response_body, output)
                        .map_err(crate::operation::start_activity_stream::StartActivityStreamError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "InvalidDBInstanceState" => crate::operation::start_activity_stream::StartActivityStreamError::InvalidDbInstanceStateFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::InvalidDbInstanceStateFaultBuilder::default();
                output =
                    crate::protocol_serde::shape_invalid_db_instance_state_fault::de_invalid_db_instance_state_fault_xml_err(_response_body, output)
                        .map_err(crate::operation::start_activity_stream::StartActivityStreamError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "KMSKeyNotAccessibleFault" => crate::operation::start_activity_stream::StartActivityStreamError::KmsKeyNotAccessibleFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::KmsKeyNotAccessibleFaultBuilder::default();
                output = crate::protocol_serde::shape_kms_key_not_accessible_fault::de_kms_key_not_accessible_fault_xml_err(_response_body, output)
                    .map_err(crate::operation::start_activity_stream::StartActivityStreamError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        "ResourceNotFoundFault" => crate::operation::start_activity_stream::StartActivityStreamError::ResourceNotFoundFault({
            #[allow(unused_mut)]
            let mut tmp = {
                #[allow(unused_mut)]
                let mut output = crate::types::error::builders::ResourceNotFoundFaultBuilder::default();
                output = crate::protocol_serde::shape_resource_not_found_fault::de_resource_not_found_fault_xml_err(_response_body, output)
                    .map_err(crate::operation::start_activity_stream::StartActivityStreamError::unhandled)?;
                let output = output.meta(generic);
                output.build()
            };
            if tmp.message.is_none() {
                tmp.message = _error_message;
            }
            tmp
        }),
        _ => crate::operation::start_activity_stream::StartActivityStreamError::generic(generic),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn de_start_activity_stream_http_response(
    _response_status: u16,
    _response_headers: &::aws_smithy_runtime_api::http::Headers,
    _response_body: &[u8],
) -> std::result::Result<
    crate::operation::start_activity_stream::StartActivityStreamOutput,
    crate::operation::start_activity_stream::StartActivityStreamError,
> {
    Ok({
        #[allow(unused_mut)]
        let mut output = crate::operation::start_activity_stream::builders::StartActivityStreamOutputBuilder::default();
        output = crate::protocol_serde::shape_start_activity_stream::de_start_activity_stream(_response_body, output)
            .map_err(crate::operation::start_activity_stream::StartActivityStreamError::unhandled)?;
        output._set_request_id(::aws_types::request_id::RequestId::request_id(_response_headers).map(str::to_string));
        output.build()
    })
}

#[allow(unused_mut)]
pub fn de_start_activity_stream(
    inp: &[u8],
    mut builder: crate::operation::start_activity_stream::builders::StartActivityStreamOutputBuilder,
) -> std::result::Result<crate::operation::start_activity_stream::builders::StartActivityStreamOutputBuilder, ::aws_smithy_xml::decode::XmlDecodeError>
{
    let mut doc = ::aws_smithy_xml::decode::Document::try_from(inp)?;

    #[allow(unused_mut)]
    let mut decoder = doc.root_element()?;
    #[allow(unused_variables)]
    let start_el = decoder.start_el();
    if !(start_el.matches("StartActivityStreamResponse")) {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
            "invalid root, expected StartActivityStreamResponse got {:?}",
            start_el
        )));
    }
    if let Some(mut result_tag) = decoder.next_tag() {
        let start_el = result_tag.start_el();
        if !(start_el.matches("StartActivityStreamResult")) {
            return Err(::aws_smithy_xml::decode::XmlDecodeError::custom(format!(
                "invalid result, expected StartActivityStreamResult got {:?}",
                start_el
            )));
        }
        while let Some(mut tag) = result_tag.next_tag() {
            match tag.start_el() {
            s if s.matches("KmsKeyId") /* KmsKeyId com.amazonaws.rds.synthetic#StartActivityStreamOutput$KmsKeyId */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_kms_key_id(var_1);
            }
            ,
            s if s.matches("KinesisStreamName") /* KinesisStreamName com.amazonaws.rds.synthetic#StartActivityStreamOutput$KinesisStreamName */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_kinesis_stream_name(var_2);
            }
            ,
            s if s.matches("Status") /* Status com.amazonaws.rds.synthetic#StartActivityStreamOutput$Status */ =>  {
                let var_3 =
                    Some(
                        Result::<crate::types::ActivityStreamStatus, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::ActivityStreamStatus::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_status(var_3);
            }
            ,
            s if s.matches("Mode") /* Mode com.amazonaws.rds.synthetic#StartActivityStreamOutput$Mode */ =>  {
                let var_4 =
                    Some(
                        Result::<crate::types::ActivityStreamMode, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::ActivityStreamMode::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_mode(var_4);
            }
            ,
            s if s.matches("ApplyImmediately") /* ApplyImmediately com.amazonaws.rds.synthetic#StartActivityStreamOutput$ApplyImmediately */ =>  {
                let var_5 =
                    Some(
                         {
                            <bool as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (boolean: `com.amazonaws.rds#Boolean`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_apply_immediately(var_5);
            }
            ,
            s if s.matches("EngineNativeAuditFieldsIncluded") /* EngineNativeAuditFieldsIncluded com.amazonaws.rds.synthetic#StartActivityStreamOutput$EngineNativeAuditFieldsIncluded */ =>  {
                let var_6 =
                    Some(
                         {
                            <bool as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (boolean: `com.amazonaws.rds#BooleanOptional`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_engine_native_audit_fields_included(var_6);
            }
            ,
            _ => {}
        }
        }
    } else {
        return Err(::aws_smithy_xml::decode::XmlDecodeError::custom("expected StartActivityStreamResult tag"));
    };
    Ok(builder)
}

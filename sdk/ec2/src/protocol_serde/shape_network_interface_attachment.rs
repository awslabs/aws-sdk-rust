// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_network_interface_attachment(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::NetworkInterfaceAttachment, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::NetworkInterfaceAttachment::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("attachTime") /* AttachTime com.amazonaws.ec2#NetworkInterfaceAttachment$AttachTime */ =>  {
                let var_1 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.ec2#DateTime`)"))
                        ?
                    )
                ;
                builder = builder.set_attach_time(var_1);
            }
            ,
            s if s.matches("attachmentId") /* AttachmentId com.amazonaws.ec2#NetworkInterfaceAttachment$AttachmentId */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_attachment_id(var_2);
            }
            ,
            s if s.matches("deleteOnTermination") /* DeleteOnTermination com.amazonaws.ec2#NetworkInterfaceAttachment$DeleteOnTermination */ =>  {
                let var_3 =
                    Some(
                         {
                            <bool as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (boolean: `com.amazonaws.ec2#Boolean`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_delete_on_termination(var_3);
            }
            ,
            s if s.matches("deviceIndex") /* DeviceIndex com.amazonaws.ec2#NetworkInterfaceAttachment$DeviceIndex */ =>  {
                let var_4 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.ec2#Integer`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_device_index(var_4);
            }
            ,
            s if s.matches("networkCardIndex") /* NetworkCardIndex com.amazonaws.ec2#NetworkInterfaceAttachment$NetworkCardIndex */ =>  {
                let var_5 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.ec2#Integer`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_network_card_index(var_5);
            }
            ,
            s if s.matches("instanceId") /* InstanceId com.amazonaws.ec2#NetworkInterfaceAttachment$InstanceId */ =>  {
                let var_6 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_instance_id(var_6);
            }
            ,
            s if s.matches("instanceOwnerId") /* InstanceOwnerId com.amazonaws.ec2#NetworkInterfaceAttachment$InstanceOwnerId */ =>  {
                let var_7 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_instance_owner_id(var_7);
            }
            ,
            s if s.matches("status") /* Status com.amazonaws.ec2#NetworkInterfaceAttachment$Status */ =>  {
                let var_8 =
                    Some(
                        Result::<crate::types::AttachmentStatus, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::AttachmentStatus::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_status(var_8);
            }
            ,
            s if s.matches("enaSrdSpecification") /* EnaSrdSpecification com.amazonaws.ec2#NetworkInterfaceAttachment$EnaSrdSpecification */ =>  {
                let var_9 =
                    Some(
                        crate::protocol_serde::shape_attachment_ena_srd_specification::de_attachment_ena_srd_specification(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_ena_srd_specification(var_9);
            }
            ,
            s if s.matches("enaQueueCount") /* EnaQueueCount com.amazonaws.ec2#NetworkInterfaceAttachment$EnaQueueCount */ =>  {
                let var_10 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.ec2#Integer`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_ena_queue_count(var_10);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

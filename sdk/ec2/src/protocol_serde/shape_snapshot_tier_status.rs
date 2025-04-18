// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_snapshot_tier_status(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::SnapshotTierStatus, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::SnapshotTierStatus::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("snapshotId") /* SnapshotId com.amazonaws.ec2#SnapshotTierStatus$SnapshotId */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_snapshot_id(var_1);
            }
            ,
            s if s.matches("volumeId") /* VolumeId com.amazonaws.ec2#SnapshotTierStatus$VolumeId */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_volume_id(var_2);
            }
            ,
            s if s.matches("status") /* Status com.amazonaws.ec2#SnapshotTierStatus$Status */ =>  {
                let var_3 =
                    Some(
                        Result::<crate::types::SnapshotState, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::SnapshotState::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_status(var_3);
            }
            ,
            s if s.matches("ownerId") /* OwnerId com.amazonaws.ec2#SnapshotTierStatus$OwnerId */ =>  {
                let var_4 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_owner_id(var_4);
            }
            ,
            s if s.matches("tagSet") /* Tags com.amazonaws.ec2#SnapshotTierStatus$Tags */ =>  {
                let var_5 =
                    Some(
                        crate::protocol_serde::shape_tag_list::de_tag_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_tags(var_5);
            }
            ,
            s if s.matches("storageTier") /* StorageTier com.amazonaws.ec2#SnapshotTierStatus$StorageTier */ =>  {
                let var_6 =
                    Some(
                        Result::<crate::types::StorageTier, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::StorageTier::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_storage_tier(var_6);
            }
            ,
            s if s.matches("lastTieringStartTime") /* LastTieringStartTime com.amazonaws.ec2#SnapshotTierStatus$LastTieringStartTime */ =>  {
                let var_7 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.ec2#MillisecondDateTime`)"))
                        ?
                    )
                ;
                builder = builder.set_last_tiering_start_time(var_7);
            }
            ,
            s if s.matches("lastTieringProgress") /* LastTieringProgress com.amazonaws.ec2#SnapshotTierStatus$LastTieringProgress */ =>  {
                let var_8 =
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
                builder = builder.set_last_tiering_progress(var_8);
            }
            ,
            s if s.matches("lastTieringOperationStatus") /* LastTieringOperationStatus com.amazonaws.ec2#SnapshotTierStatus$LastTieringOperationStatus */ =>  {
                let var_9 =
                    Some(
                        Result::<crate::types::TieringOperationStatus, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::TieringOperationStatus::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_last_tiering_operation_status(var_9);
            }
            ,
            s if s.matches("lastTieringOperationStatusDetail") /* LastTieringOperationStatusDetail com.amazonaws.ec2#SnapshotTierStatus$LastTieringOperationStatusDetail */ =>  {
                let var_10 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_last_tiering_operation_status_detail(var_10);
            }
            ,
            s if s.matches("archivalCompleteTime") /* ArchivalCompleteTime com.amazonaws.ec2#SnapshotTierStatus$ArchivalCompleteTime */ =>  {
                let var_11 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.ec2#MillisecondDateTime`)"))
                        ?
                    )
                ;
                builder = builder.set_archival_complete_time(var_11);
            }
            ,
            s if s.matches("restoreExpiryTime") /* RestoreExpiryTime com.amazonaws.ec2#SnapshotTierStatus$RestoreExpiryTime */ =>  {
                let var_12 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.ec2#MillisecondDateTime`)"))
                        ?
                    )
                ;
                builder = builder.set_restore_expiry_time(var_12);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

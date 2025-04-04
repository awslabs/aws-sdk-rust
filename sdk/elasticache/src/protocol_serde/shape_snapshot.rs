// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_snapshot(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::Snapshot, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::Snapshot::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("SnapshotName") /* SnapshotName com.amazonaws.elasticache#Snapshot$SnapshotName */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_snapshot_name(var_1);
            }
            ,
            s if s.matches("ReplicationGroupId") /* ReplicationGroupId com.amazonaws.elasticache#Snapshot$ReplicationGroupId */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_replication_group_id(var_2);
            }
            ,
            s if s.matches("ReplicationGroupDescription") /* ReplicationGroupDescription com.amazonaws.elasticache#Snapshot$ReplicationGroupDescription */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_replication_group_description(var_3);
            }
            ,
            s if s.matches("CacheClusterId") /* CacheClusterId com.amazonaws.elasticache#Snapshot$CacheClusterId */ =>  {
                let var_4 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_cache_cluster_id(var_4);
            }
            ,
            s if s.matches("SnapshotStatus") /* SnapshotStatus com.amazonaws.elasticache#Snapshot$SnapshotStatus */ =>  {
                let var_5 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_snapshot_status(var_5);
            }
            ,
            s if s.matches("SnapshotSource") /* SnapshotSource com.amazonaws.elasticache#Snapshot$SnapshotSource */ =>  {
                let var_6 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_snapshot_source(var_6);
            }
            ,
            s if s.matches("CacheNodeType") /* CacheNodeType com.amazonaws.elasticache#Snapshot$CacheNodeType */ =>  {
                let var_7 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_cache_node_type(var_7);
            }
            ,
            s if s.matches("Engine") /* Engine com.amazonaws.elasticache#Snapshot$Engine */ =>  {
                let var_8 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_engine(var_8);
            }
            ,
            s if s.matches("EngineVersion") /* EngineVersion com.amazonaws.elasticache#Snapshot$EngineVersion */ =>  {
                let var_9 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_engine_version(var_9);
            }
            ,
            s if s.matches("NumCacheNodes") /* NumCacheNodes com.amazonaws.elasticache#Snapshot$NumCacheNodes */ =>  {
                let var_10 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.elasticache#IntegerOptional`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_num_cache_nodes(var_10);
            }
            ,
            s if s.matches("PreferredAvailabilityZone") /* PreferredAvailabilityZone com.amazonaws.elasticache#Snapshot$PreferredAvailabilityZone */ =>  {
                let var_11 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_preferred_availability_zone(var_11);
            }
            ,
            s if s.matches("PreferredOutpostArn") /* PreferredOutpostArn com.amazonaws.elasticache#Snapshot$PreferredOutpostArn */ =>  {
                let var_12 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_preferred_outpost_arn(var_12);
            }
            ,
            s if s.matches("CacheClusterCreateTime") /* CacheClusterCreateTime com.amazonaws.elasticache#Snapshot$CacheClusterCreateTime */ =>  {
                let var_13 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.elasticache#TStamp`)"))
                        ?
                    )
                ;
                builder = builder.set_cache_cluster_create_time(var_13);
            }
            ,
            s if s.matches("PreferredMaintenanceWindow") /* PreferredMaintenanceWindow com.amazonaws.elasticache#Snapshot$PreferredMaintenanceWindow */ =>  {
                let var_14 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_preferred_maintenance_window(var_14);
            }
            ,
            s if s.matches("TopicArn") /* TopicArn com.amazonaws.elasticache#Snapshot$TopicArn */ =>  {
                let var_15 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_topic_arn(var_15);
            }
            ,
            s if s.matches("Port") /* Port com.amazonaws.elasticache#Snapshot$Port */ =>  {
                let var_16 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.elasticache#IntegerOptional`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_port(var_16);
            }
            ,
            s if s.matches("CacheParameterGroupName") /* CacheParameterGroupName com.amazonaws.elasticache#Snapshot$CacheParameterGroupName */ =>  {
                let var_17 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_cache_parameter_group_name(var_17);
            }
            ,
            s if s.matches("CacheSubnetGroupName") /* CacheSubnetGroupName com.amazonaws.elasticache#Snapshot$CacheSubnetGroupName */ =>  {
                let var_18 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_cache_subnet_group_name(var_18);
            }
            ,
            s if s.matches("VpcId") /* VpcId com.amazonaws.elasticache#Snapshot$VpcId */ =>  {
                let var_19 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_vpc_id(var_19);
            }
            ,
            s if s.matches("AutoMinorVersionUpgrade") /* AutoMinorVersionUpgrade com.amazonaws.elasticache#Snapshot$AutoMinorVersionUpgrade */ =>  {
                let var_20 =
                    Some(
                         {
                            <bool as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (boolean: `com.amazonaws.elasticache#Boolean`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_auto_minor_version_upgrade(var_20);
            }
            ,
            s if s.matches("SnapshotRetentionLimit") /* SnapshotRetentionLimit com.amazonaws.elasticache#Snapshot$SnapshotRetentionLimit */ =>  {
                let var_21 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.elasticache#IntegerOptional`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_snapshot_retention_limit(var_21);
            }
            ,
            s if s.matches("SnapshotWindow") /* SnapshotWindow com.amazonaws.elasticache#Snapshot$SnapshotWindow */ =>  {
                let var_22 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_snapshot_window(var_22);
            }
            ,
            s if s.matches("NumNodeGroups") /* NumNodeGroups com.amazonaws.elasticache#Snapshot$NumNodeGroups */ =>  {
                let var_23 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.elasticache#IntegerOptional`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_num_node_groups(var_23);
            }
            ,
            s if s.matches("AutomaticFailover") /* AutomaticFailover com.amazonaws.elasticache#Snapshot$AutomaticFailover */ =>  {
                let var_24 =
                    Some(
                        Result::<crate::types::AutomaticFailoverStatus, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::AutomaticFailoverStatus::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_automatic_failover(var_24);
            }
            ,
            s if s.matches("NodeSnapshots") /* NodeSnapshots com.amazonaws.elasticache#Snapshot$NodeSnapshots */ =>  {
                let var_25 =
                    Some(
                        crate::protocol_serde::shape_node_snapshot_list::de_node_snapshot_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_node_snapshots(var_25);
            }
            ,
            s if s.matches("KmsKeyId") /* KmsKeyId com.amazonaws.elasticache#Snapshot$KmsKeyId */ =>  {
                let var_26 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_kms_key_id(var_26);
            }
            ,
            s if s.matches("ARN") /* ARN com.amazonaws.elasticache#Snapshot$ARN */ =>  {
                let var_27 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_arn(var_27);
            }
            ,
            s if s.matches("DataTiering") /* DataTiering com.amazonaws.elasticache#Snapshot$DataTiering */ =>  {
                let var_28 =
                    Some(
                        Result::<crate::types::DataTieringStatus, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::DataTieringStatus::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_data_tiering(var_28);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

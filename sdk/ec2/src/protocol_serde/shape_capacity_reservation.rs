// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_capacity_reservation(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::CapacityReservation, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::CapacityReservation::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("capacityReservationId") /* CapacityReservationId com.amazonaws.ec2#CapacityReservation$CapacityReservationId */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_capacity_reservation_id(var_1);
            }
            ,
            s if s.matches("ownerId") /* OwnerId com.amazonaws.ec2#CapacityReservation$OwnerId */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_owner_id(var_2);
            }
            ,
            s if s.matches("capacityReservationArn") /* CapacityReservationArn com.amazonaws.ec2#CapacityReservation$CapacityReservationArn */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_capacity_reservation_arn(var_3);
            }
            ,
            s if s.matches("availabilityZoneId") /* AvailabilityZoneId com.amazonaws.ec2#CapacityReservation$AvailabilityZoneId */ =>  {
                let var_4 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_availability_zone_id(var_4);
            }
            ,
            s if s.matches("instanceType") /* InstanceType com.amazonaws.ec2#CapacityReservation$InstanceType */ =>  {
                let var_5 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_instance_type(var_5);
            }
            ,
            s if s.matches("instancePlatform") /* InstancePlatform com.amazonaws.ec2#CapacityReservation$InstancePlatform */ =>  {
                let var_6 =
                    Some(
                        Result::<crate::types::CapacityReservationInstancePlatform, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::CapacityReservationInstancePlatform::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_instance_platform(var_6);
            }
            ,
            s if s.matches("availabilityZone") /* AvailabilityZone com.amazonaws.ec2#CapacityReservation$AvailabilityZone */ =>  {
                let var_7 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_availability_zone(var_7);
            }
            ,
            s if s.matches("tenancy") /* Tenancy com.amazonaws.ec2#CapacityReservation$Tenancy */ =>  {
                let var_8 =
                    Some(
                        Result::<crate::types::CapacityReservationTenancy, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::CapacityReservationTenancy::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_tenancy(var_8);
            }
            ,
            s if s.matches("totalInstanceCount") /* TotalInstanceCount com.amazonaws.ec2#CapacityReservation$TotalInstanceCount */ =>  {
                let var_9 =
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
                builder = builder.set_total_instance_count(var_9);
            }
            ,
            s if s.matches("availableInstanceCount") /* AvailableInstanceCount com.amazonaws.ec2#CapacityReservation$AvailableInstanceCount */ =>  {
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
                builder = builder.set_available_instance_count(var_10);
            }
            ,
            s if s.matches("ebsOptimized") /* EbsOptimized com.amazonaws.ec2#CapacityReservation$EbsOptimized */ =>  {
                let var_11 =
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
                builder = builder.set_ebs_optimized(var_11);
            }
            ,
            s if s.matches("ephemeralStorage") /* EphemeralStorage com.amazonaws.ec2#CapacityReservation$EphemeralStorage */ =>  {
                let var_12 =
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
                builder = builder.set_ephemeral_storage(var_12);
            }
            ,
            s if s.matches("state") /* State com.amazonaws.ec2#CapacityReservation$State */ =>  {
                let var_13 =
                    Some(
                        Result::<crate::types::CapacityReservationState, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::CapacityReservationState::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_state(var_13);
            }
            ,
            s if s.matches("startDate") /* StartDate com.amazonaws.ec2#CapacityReservation$StartDate */ =>  {
                let var_14 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.ec2#MillisecondDateTime`)"))
                        ?
                    )
                ;
                builder = builder.set_start_date(var_14);
            }
            ,
            s if s.matches("endDate") /* EndDate com.amazonaws.ec2#CapacityReservation$EndDate */ =>  {
                let var_15 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.ec2#DateTime`)"))
                        ?
                    )
                ;
                builder = builder.set_end_date(var_15);
            }
            ,
            s if s.matches("endDateType") /* EndDateType com.amazonaws.ec2#CapacityReservation$EndDateType */ =>  {
                let var_16 =
                    Some(
                        Result::<crate::types::EndDateType, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::EndDateType::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_end_date_type(var_16);
            }
            ,
            s if s.matches("instanceMatchCriteria") /* InstanceMatchCriteria com.amazonaws.ec2#CapacityReservation$InstanceMatchCriteria */ =>  {
                let var_17 =
                    Some(
                        Result::<crate::types::InstanceMatchCriteria, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::InstanceMatchCriteria::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_instance_match_criteria(var_17);
            }
            ,
            s if s.matches("createDate") /* CreateDate com.amazonaws.ec2#CapacityReservation$CreateDate */ =>  {
                let var_18 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.ec2#DateTime`)"))
                        ?
                    )
                ;
                builder = builder.set_create_date(var_18);
            }
            ,
            s if s.matches("tagSet") /* Tags com.amazonaws.ec2#CapacityReservation$Tags */ =>  {
                let var_19 =
                    Some(
                        crate::protocol_serde::shape_tag_list::de_tag_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_tags(var_19);
            }
            ,
            s if s.matches("outpostArn") /* OutpostArn com.amazonaws.ec2#CapacityReservation$OutpostArn */ =>  {
                let var_20 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_outpost_arn(var_20);
            }
            ,
            s if s.matches("capacityReservationFleetId") /* CapacityReservationFleetId com.amazonaws.ec2#CapacityReservation$CapacityReservationFleetId */ =>  {
                let var_21 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_capacity_reservation_fleet_id(var_21);
            }
            ,
            s if s.matches("placementGroupArn") /* PlacementGroupArn com.amazonaws.ec2#CapacityReservation$PlacementGroupArn */ =>  {
                let var_22 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_placement_group_arn(var_22);
            }
            ,
            s if s.matches("capacityAllocationSet") /* CapacityAllocations com.amazonaws.ec2#CapacityReservation$CapacityAllocations */ =>  {
                let var_23 =
                    Some(
                        crate::protocol_serde::shape_capacity_allocations::de_capacity_allocations(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_capacity_allocations(var_23);
            }
            ,
            s if s.matches("reservationType") /* ReservationType com.amazonaws.ec2#CapacityReservation$ReservationType */ =>  {
                let var_24 =
                    Some(
                        Result::<crate::types::CapacityReservationType, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::CapacityReservationType::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_reservation_type(var_24);
            }
            ,
            s if s.matches("unusedReservationBillingOwnerId") /* UnusedReservationBillingOwnerId com.amazonaws.ec2#CapacityReservation$UnusedReservationBillingOwnerId */ =>  {
                let var_25 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_unused_reservation_billing_owner_id(var_25);
            }
            ,
            s if s.matches("commitmentInfo") /* CommitmentInfo com.amazonaws.ec2#CapacityReservation$CommitmentInfo */ =>  {
                let var_26 =
                    Some(
                        crate::protocol_serde::shape_capacity_reservation_commitment_info::de_capacity_reservation_commitment_info(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_commitment_info(var_26);
            }
            ,
            s if s.matches("deliveryPreference") /* DeliveryPreference com.amazonaws.ec2#CapacityReservation$DeliveryPreference */ =>  {
                let var_27 =
                    Some(
                        Result::<crate::types::CapacityReservationDeliveryPreference, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::CapacityReservationDeliveryPreference::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_delivery_preference(var_27);
            }
            ,
            s if s.matches("capacityBlockId") /* CapacityBlockId com.amazonaws.ec2#CapacityReservation$CapacityBlockId */ =>  {
                let var_28 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_capacity_block_id(var_28);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

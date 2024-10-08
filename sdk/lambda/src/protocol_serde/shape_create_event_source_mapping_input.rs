// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_event_source_mapping_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_event_source_mapping::CreateEventSourceMappingInput,
) -> Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.amazon_managed_kafka_event_source_config {
        #[allow(unused_mut)]
        let mut object_2 = object.key("AmazonManagedKafkaEventSourceConfig").start_object();
        crate::protocol_serde::shape_amazon_managed_kafka_event_source_config::ser_amazon_managed_kafka_event_source_config(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.batch_size {
        object.key("BatchSize").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_3).into()),
        );
    }
    if let Some(var_4) = &input.bisect_batch_on_function_error {
        object.key("BisectBatchOnFunctionError").boolean(*var_4);
    }
    if let Some(var_5) = &input.destination_config {
        #[allow(unused_mut)]
        let mut object_6 = object.key("DestinationConfig").start_object();
        crate::protocol_serde::shape_destination_config::ser_destination_config(&mut object_6, var_5)?;
        object_6.finish();
    }
    if let Some(var_7) = &input.document_db_event_source_config {
        #[allow(unused_mut)]
        let mut object_8 = object.key("DocumentDBEventSourceConfig").start_object();
        crate::protocol_serde::shape_document_db_event_source_config::ser_document_db_event_source_config(&mut object_8, var_7)?;
        object_8.finish();
    }
    if let Some(var_9) = &input.enabled {
        object.key("Enabled").boolean(*var_9);
    }
    if let Some(var_10) = &input.event_source_arn {
        object.key("EventSourceArn").string(var_10.as_str());
    }
    if let Some(var_11) = &input.filter_criteria {
        #[allow(unused_mut)]
        let mut object_12 = object.key("FilterCriteria").start_object();
        crate::protocol_serde::shape_filter_criteria::ser_filter_criteria(&mut object_12, var_11)?;
        object_12.finish();
    }
    if let Some(var_13) = &input.function_name {
        object.key("FunctionName").string(var_13.as_str());
    }
    if let Some(var_14) = &input.function_response_types {
        let mut array_15 = object.key("FunctionResponseTypes").start_array();
        for item_16 in var_14 {
            {
                array_15.value().string(item_16.as_str());
            }
        }
        array_15.finish();
    }
    if let Some(var_17) = &input.kms_key_arn {
        object.key("KMSKeyArn").string(var_17.as_str());
    }
    if let Some(var_18) = &input.maximum_batching_window_in_seconds {
        object.key("MaximumBatchingWindowInSeconds").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_18).into()),
        );
    }
    if let Some(var_19) = &input.maximum_record_age_in_seconds {
        object.key("MaximumRecordAgeInSeconds").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_19).into()),
        );
    }
    if let Some(var_20) = &input.maximum_retry_attempts {
        object.key("MaximumRetryAttempts").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_20).into()),
        );
    }
    if let Some(var_21) = &input.parallelization_factor {
        object.key("ParallelizationFactor").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_21).into()),
        );
    }
    if let Some(var_22) = &input.queues {
        let mut array_23 = object.key("Queues").start_array();
        for item_24 in var_22 {
            {
                array_23.value().string(item_24.as_str());
            }
        }
        array_23.finish();
    }
    if let Some(var_25) = &input.scaling_config {
        #[allow(unused_mut)]
        let mut object_26 = object.key("ScalingConfig").start_object();
        crate::protocol_serde::shape_scaling_config::ser_scaling_config(&mut object_26, var_25)?;
        object_26.finish();
    }
    if let Some(var_27) = &input.self_managed_event_source {
        #[allow(unused_mut)]
        let mut object_28 = object.key("SelfManagedEventSource").start_object();
        crate::protocol_serde::shape_self_managed_event_source::ser_self_managed_event_source(&mut object_28, var_27)?;
        object_28.finish();
    }
    if let Some(var_29) = &input.self_managed_kafka_event_source_config {
        #[allow(unused_mut)]
        let mut object_30 = object.key("SelfManagedKafkaEventSourceConfig").start_object();
        crate::protocol_serde::shape_self_managed_kafka_event_source_config::ser_self_managed_kafka_event_source_config(&mut object_30, var_29)?;
        object_30.finish();
    }
    if let Some(var_31) = &input.source_access_configurations {
        let mut array_32 = object.key("SourceAccessConfigurations").start_array();
        for item_33 in var_31 {
            {
                #[allow(unused_mut)]
                let mut object_34 = array_32.value().start_object();
                crate::protocol_serde::shape_source_access_configuration::ser_source_access_configuration(&mut object_34, item_33)?;
                object_34.finish();
            }
        }
        array_32.finish();
    }
    if let Some(var_35) = &input.starting_position {
        object.key("StartingPosition").string(var_35.as_str());
    }
    if let Some(var_36) = &input.starting_position_timestamp {
        object
            .key("StartingPositionTimestamp")
            .date_time(var_36, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    if let Some(var_37) = &input.tags {
        #[allow(unused_mut)]
        let mut object_38 = object.key("Tags").start_object();
        for (key_39, value_40) in var_37 {
            {
                object_38.key(key_39.as_str()).string(value_40.as_str());
            }
        }
        object_38.finish();
    }
    if let Some(var_41) = &input.topics {
        let mut array_42 = object.key("Topics").start_array();
        for item_43 in var_41 {
            {
                array_42.value().string(item_43.as_str());
            }
        }
        array_42.finish();
    }
    if let Some(var_44) = &input.tumbling_window_in_seconds {
        object.key("TumblingWindowInSeconds").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_44).into()),
        );
    }
    Ok(())
}

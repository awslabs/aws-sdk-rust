// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_describe_applicable_individual_assessments_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::describe_applicable_individual_assessments::DescribeApplicableIndividualAssessmentsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.replication_task_arn {
        object.key("ReplicationTaskArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.replication_instance_arn {
        object.key("ReplicationInstanceArn").string(var_2.as_str());
    }
    if let Some(var_3) = &input.replication_config_arn {
        object.key("ReplicationConfigArn").string(var_3.as_str());
    }
    if let Some(var_4) = &input.source_engine_name {
        object.key("SourceEngineName").string(var_4.as_str());
    }
    if let Some(var_5) = &input.target_engine_name {
        object.key("TargetEngineName").string(var_5.as_str());
    }
    if let Some(var_6) = &input.migration_type {
        object.key("MigrationType").string(var_6.as_str());
    }
    if let Some(var_7) = &input.max_records {
        object.key("MaxRecords").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_7).into()),
        );
    }
    if let Some(var_8) = &input.marker {
        object.key("Marker").string(var_8.as_str());
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_put_configuration_aggregator_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::put_configuration_aggregator::PutConfigurationAggregatorInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.configuration_aggregator_name {
        object.key("ConfigurationAggregatorName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.account_aggregation_sources {
        let mut array_3 = object.key("AccountAggregationSources").start_array();
        for item_4 in var_2 {
            {
                #[allow(unused_mut)]
                let mut object_5 = array_3.value().start_object();
                crate::protocol_serde::shape_account_aggregation_source::ser_account_aggregation_source(&mut object_5, item_4)?;
                object_5.finish();
            }
        }
        array_3.finish();
    }
    if let Some(var_6) = &input.organization_aggregation_source {
        #[allow(unused_mut)]
        let mut object_7 = object.key("OrganizationAggregationSource").start_object();
        crate::protocol_serde::shape_organization_aggregation_source::ser_organization_aggregation_source(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.tags {
        let mut array_9 = object.key("Tags").start_array();
        for item_10 in var_8 {
            {
                #[allow(unused_mut)]
                let mut object_11 = array_9.value().start_object();
                crate::protocol_serde::shape_tag::ser_tag(&mut object_11, item_10)?;
                object_11.finish();
            }
        }
        array_9.finish();
    }
    if let Some(var_12) = &input.aggregator_filters {
        #[allow(unused_mut)]
        let mut object_13 = object.key("AggregatorFilters").start_object();
        crate::protocol_serde::shape_aggregator_filters::ser_aggregator_filters(&mut object_13, var_12)?;
        object_13.finish();
    }
    Ok(())
}

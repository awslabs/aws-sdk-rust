// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn reflens_describe_configuration_templates_output_output_next_token(
    input: &crate::operation::describe_configuration_templates::DescribeConfigurationTemplatesOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_describe_deliveries_output_output_next_token(
    input: &crate::operation::describe_deliveries::DescribeDeliveriesOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_describe_delivery_destinations_output_output_next_token(
    input: &crate::operation::describe_delivery_destinations::DescribeDeliveryDestinationsOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_describe_delivery_sources_output_output_next_token(
    input: &crate::operation::describe_delivery_sources::DescribeDeliverySourcesOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_describe_destinations_output_output_next_token(
    input: &crate::operation::describe_destinations::DescribeDestinationsOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_describe_log_groups_output_output_next_token(
    input: &crate::operation::describe_log_groups::DescribeLogGroupsOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_describe_log_streams_output_output_next_token(
    input: &crate::operation::describe_log_streams::DescribeLogStreamsOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_describe_metric_filters_output_output_next_token(
    input: &crate::operation::describe_metric_filters::DescribeMetricFiltersOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_describe_subscription_filters_output_output_next_token(
    input: &crate::operation::describe_subscription_filters::DescribeSubscriptionFiltersOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_filter_log_events_output_output_next_token(
    input: &crate::operation::filter_log_events::FilterLogEventsOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_get_log_events_output_output_next_forward_token(
    input: &crate::operation::get_log_events::GetLogEventsOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_forward_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_list_anomalies_output_output_next_token(
    input: &crate::operation::list_anomalies::ListAnomaliesOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_list_log_anomaly_detectors_output_output_next_token(
    input: &crate::operation::list_log_anomaly_detectors::ListLogAnomalyDetectorsOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn reflens_list_log_groups_for_query_output_output_next_token(
    input: &crate::operation::list_log_groups_for_query::ListLogGroupsForQueryOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_describe_configuration_templates_output_output_configuration_templates(
    input: crate::operation::describe_configuration_templates::DescribeConfigurationTemplatesOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::ConfigurationTemplate>> {
    let input = input.configuration_templates?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_describe_deliveries_output_output_deliveries(
    input: crate::operation::describe_deliveries::DescribeDeliveriesOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::Delivery>> {
    let input = input.deliveries?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_describe_delivery_destinations_output_output_delivery_destinations(
    input: crate::operation::describe_delivery_destinations::DescribeDeliveryDestinationsOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::DeliveryDestination>> {
    let input = input.delivery_destinations?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_describe_delivery_sources_output_output_delivery_sources(
    input: crate::operation::describe_delivery_sources::DescribeDeliverySourcesOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::DeliverySource>> {
    let input = input.delivery_sources?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_describe_destinations_output_output_destinations(
    input: crate::operation::describe_destinations::DescribeDestinationsOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::Destination>> {
    let input = input.destinations?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_describe_log_groups_output_output_log_groups(
    input: crate::operation::describe_log_groups::DescribeLogGroupsOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::LogGroup>> {
    let input = input.log_groups?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_describe_log_streams_output_output_log_streams(
    input: crate::operation::describe_log_streams::DescribeLogStreamsOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::LogStream>> {
    let input = input.log_streams?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_describe_metric_filters_output_output_metric_filters(
    input: crate::operation::describe_metric_filters::DescribeMetricFiltersOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::MetricFilter>> {
    let input = input.metric_filters?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_describe_subscription_filters_output_output_subscription_filters(
    input: crate::operation::describe_subscription_filters::DescribeSubscriptionFiltersOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::SubscriptionFilter>> {
    let input = input.subscription_filters?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_get_log_events_output_output_events(
    input: crate::operation::get_log_events::GetLogEventsOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::OutputLogEvent>> {
    let input = input.events?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_list_anomalies_output_output_anomalies(
    input: crate::operation::list_anomalies::ListAnomaliesOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::Anomaly>> {
    let input = input.anomalies?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_list_log_anomaly_detectors_output_output_anomaly_detectors(
    input: crate::operation::list_log_anomaly_detectors::ListLogAnomalyDetectorsOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::AnomalyDetector>> {
    let input = input.anomaly_detectors?;
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_list_log_groups_for_query_output_output_log_group_identifiers(
    input: crate::operation::list_log_groups_for_query::ListLogGroupsForQueryOutput,
) -> ::std::option::Option<::std::vec::Vec<::std::string::String>> {
    let input = input.log_group_identifiers?;
    ::std::option::Option::Some(input)
}

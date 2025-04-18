// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_put_metric_stream_input_input_input(
    input: &crate::operation::put_metric_stream::PutMetricStreamInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "PutMetricStream", "2010-08-01");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("Name");
    if let Some(var_2) = &input.name {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("IncludeFilters");
    if let Some(var_4) = &input.include_filters {
        let mut list_6 = scope_3.start_list(false, None);
        for item_5 in var_4 {
            #[allow(unused_mut)]
            let mut entry_7 = list_6.entry();
            crate::protocol_serde::shape_metric_stream_filter::ser_metric_stream_filter(entry_7, item_5)?;
        }
        list_6.finish();
    }
    #[allow(unused_mut)]
    let mut scope_8 = writer.prefix("ExcludeFilters");
    if let Some(var_9) = &input.exclude_filters {
        let mut list_11 = scope_8.start_list(false, None);
        for item_10 in var_9 {
            #[allow(unused_mut)]
            let mut entry_12 = list_11.entry();
            crate::protocol_serde::shape_metric_stream_filter::ser_metric_stream_filter(entry_12, item_10)?;
        }
        list_11.finish();
    }
    #[allow(unused_mut)]
    let mut scope_13 = writer.prefix("FirehoseArn");
    if let Some(var_14) = &input.firehose_arn {
        scope_13.string(var_14);
    }
    #[allow(unused_mut)]
    let mut scope_15 = writer.prefix("RoleArn");
    if let Some(var_16) = &input.role_arn {
        scope_15.string(var_16);
    }
    #[allow(unused_mut)]
    let mut scope_17 = writer.prefix("OutputFormat");
    if let Some(var_18) = &input.output_format {
        scope_17.string(var_18.as_str());
    }
    #[allow(unused_mut)]
    let mut scope_19 = writer.prefix("Tags");
    if let Some(var_20) = &input.tags {
        let mut list_22 = scope_19.start_list(false, None);
        for item_21 in var_20 {
            #[allow(unused_mut)]
            let mut entry_23 = list_22.entry();
            crate::protocol_serde::shape_tag::ser_tag(entry_23, item_21)?;
        }
        list_22.finish();
    }
    #[allow(unused_mut)]
    let mut scope_24 = writer.prefix("StatisticsConfigurations");
    if let Some(var_25) = &input.statistics_configurations {
        let mut list_27 = scope_24.start_list(false, None);
        for item_26 in var_25 {
            #[allow(unused_mut)]
            let mut entry_28 = list_27.entry();
            crate::protocol_serde::shape_metric_stream_statistics_configuration::ser_metric_stream_statistics_configuration(entry_28, item_26)?;
        }
        list_27.finish();
    }
    #[allow(unused_mut)]
    let mut scope_29 = writer.prefix("IncludeLinkedAccountsMetrics");
    if let Some(var_30) = &input.include_linked_accounts_metrics {
        scope_29.boolean(*var_30);
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

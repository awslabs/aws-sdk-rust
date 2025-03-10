// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_describe_scheduled_instance_availability_input_input_input(
    input: &crate::operation::describe_scheduled_instance_availability::DescribeScheduledInstanceAvailabilityInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "DescribeScheduledInstanceAvailability", "2016-11-15");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("DryRun");
    if let Some(var_2) = &input.dry_run {
        scope_1.boolean(*var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("Filter");
    if let Some(var_4) = &input.filters {
        if !var_4.is_empty() {
            let mut list_6 = scope_3.start_list(true, Some("Filter"));
            for item_5 in var_4 {
                #[allow(unused_mut)]
                let mut entry_7 = list_6.entry();
                crate::protocol_serde::shape_filter::ser_filter(entry_7, item_5)?;
            }
            list_6.finish();
        }
    }
    #[allow(unused_mut)]
    let mut scope_8 = writer.prefix("FirstSlotStartTimeRange");
    if let Some(var_9) = &input.first_slot_start_time_range {
        crate::protocol_serde::shape_slot_date_time_range_request::ser_slot_date_time_range_request(scope_8, var_9)?;
    }
    #[allow(unused_mut)]
    let mut scope_10 = writer.prefix("MaxResults");
    if let Some(var_11) = &input.max_results {
        scope_10.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_11).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_12 = writer.prefix("MaxSlotDurationInHours");
    if let Some(var_13) = &input.max_slot_duration_in_hours {
        scope_12.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_13).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_14 = writer.prefix("MinSlotDurationInHours");
    if let Some(var_15) = &input.min_slot_duration_in_hours {
        scope_14.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_15).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_16 = writer.prefix("NextToken");
    if let Some(var_17) = &input.next_token {
        scope_16.string(var_17);
    }
    #[allow(unused_mut)]
    let mut scope_18 = writer.prefix("Recurrence");
    if let Some(var_19) = &input.recurrence {
        crate::protocol_serde::shape_scheduled_instance_recurrence_request::ser_scheduled_instance_recurrence_request(scope_18, var_19)?;
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_list_test_grid_sessions_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::list_test_grid_sessions::ListTestGridSessionsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.project_arn {
        object.key("projectArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.status {
        object.key("status").string(var_2.as_str());
    }
    if let Some(var_3) = &input.creation_time_after {
        object
            .key("creationTimeAfter")
            .date_time(var_3, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    if let Some(var_4) = &input.creation_time_before {
        object
            .key("creationTimeBefore")
            .date_time(var_4, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    if let Some(var_5) = &input.end_time_after {
        object
            .key("endTimeAfter")
            .date_time(var_5, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    if let Some(var_6) = &input.end_time_before {
        object
            .key("endTimeBefore")
            .date_time(var_6, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    if let Some(var_7) = &input.max_result {
        object.key("maxResult").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_7).into()),
        );
    }
    if let Some(var_8) = &input.next_token {
        object.key("nextToken").string(var_8.as_str());
    }
    Ok(())
}

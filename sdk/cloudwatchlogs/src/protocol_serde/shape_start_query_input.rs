// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_start_query_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::start_query::StartQueryInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.query_language {
        object.key("queryLanguage").string(var_1.as_str());
    }
    if let Some(var_2) = &input.log_group_name {
        object.key("logGroupName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.log_group_names {
        let mut array_4 = object.key("logGroupNames").start_array();
        for item_5 in var_3 {
            {
                array_4.value().string(item_5.as_str());
            }
        }
        array_4.finish();
    }
    if let Some(var_6) = &input.log_group_identifiers {
        let mut array_7 = object.key("logGroupIdentifiers").start_array();
        for item_8 in var_6 {
            {
                array_7.value().string(item_8.as_str());
            }
        }
        array_7.finish();
    }
    if let Some(var_9) = &input.start_time {
        object.key("startTime").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_9).into()),
        );
    }
    if let Some(var_10) = &input.end_time {
        object.key("endTime").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_10).into()),
        );
    }
    if let Some(var_11) = &input.query_string {
        object.key("queryString").string(var_11.as_str());
    }
    if let Some(var_12) = &input.limit {
        object.key("limit").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_12).into()),
        );
    }
    Ok(())
}

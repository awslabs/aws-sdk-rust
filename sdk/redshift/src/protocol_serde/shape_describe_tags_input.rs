// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_describe_tags_input_input_input(
    input: &crate::operation::describe_tags::DescribeTagsInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "DescribeTags", "2012-12-01");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("ResourceName");
    if let Some(var_2) = &input.resource_name {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("ResourceType");
    if let Some(var_4) = &input.resource_type {
        scope_3.string(var_4);
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("MaxRecords");
    if let Some(var_6) = &input.max_records {
        scope_5.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_6).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_7 = writer.prefix("Marker");
    if let Some(var_8) = &input.marker {
        scope_7.string(var_8);
    }
    #[allow(unused_mut)]
    let mut scope_9 = writer.prefix("TagKeys");
    if let Some(var_10) = &input.tag_keys {
        let mut list_12 = scope_9.start_list(false, Some("TagKey"));
        for item_11 in var_10 {
            #[allow(unused_mut)]
            let mut entry_13 = list_12.entry();
            entry_13.string(item_11);
        }
        list_12.finish();
    }
    #[allow(unused_mut)]
    let mut scope_14 = writer.prefix("TagValues");
    if let Some(var_15) = &input.tag_values {
        let mut list_17 = scope_14.start_list(false, Some("TagValue"));
        for item_16 in var_15 {
            #[allow(unused_mut)]
            let mut entry_18 = list_17.entry();
            entry_18.string(item_16);
        }
        list_17.finish();
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

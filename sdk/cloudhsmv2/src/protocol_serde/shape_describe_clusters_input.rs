// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_describe_clusters_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::describe_clusters::DescribeClustersInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.filters {
        #[allow(unused_mut)]
        let mut object_2 = object.key("Filters").start_object();
        for (key_3, value_4) in var_1 {
            {
                let mut array_5 = object_2.key(key_3.as_str()).start_array();
                for item_6 in value_4 {
                    {
                        array_5.value().string(item_6.as_str());
                    }
                }
                array_5.finish();
            }
        }
        object_2.finish();
    }
    if let Some(var_7) = &input.next_token {
        object.key("NextToken").string(var_7.as_str());
    }
    if let Some(var_8) = &input.max_results {
        object.key("MaxResults").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_8).into()),
        );
    }
    Ok(())
}

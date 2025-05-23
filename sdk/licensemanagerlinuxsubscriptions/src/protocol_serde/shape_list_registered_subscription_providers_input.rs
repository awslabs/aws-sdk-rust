// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_list_registered_subscription_providers_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::list_registered_subscription_providers::ListRegisteredSubscriptionProvidersInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.max_results {
        object.key("MaxResults").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_1).into()),
        );
    }
    if let Some(var_2) = &input.next_token {
        object.key("NextToken").string(var_2.as_str());
    }
    if let Some(var_3) = &input.subscription_provider_sources {
        let mut array_4 = object.key("SubscriptionProviderSources").start_array();
        for item_5 in var_3 {
            {
                array_4.value().string(item_5.as_str());
            }
        }
        array_4.finish();
    }
    Ok(())
}

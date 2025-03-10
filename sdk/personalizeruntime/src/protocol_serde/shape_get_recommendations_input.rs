// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_get_recommendations_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::get_recommendations::GetRecommendationsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.campaign_arn {
        object.key("campaignArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.context {
        #[allow(unused_mut)]
        let mut object_3 = object.key("context").start_object();
        for (key_4, value_5) in var_2 {
            {
                object_3.key(key_4.as_str()).string(value_5.as_str());
            }
        }
        object_3.finish();
    }
    if let Some(var_6) = &input.filter_arn {
        object.key("filterArn").string(var_6.as_str());
    }
    if let Some(var_7) = &input.filter_values {
        #[allow(unused_mut)]
        let mut object_8 = object.key("filterValues").start_object();
        for (key_9, value_10) in var_7 {
            {
                object_8.key(key_9.as_str()).string(value_10.as_str());
            }
        }
        object_8.finish();
    }
    if let Some(var_11) = &input.item_id {
        object.key("itemId").string(var_11.as_str());
    }
    if let Some(var_12) = &input.metadata_columns {
        #[allow(unused_mut)]
        let mut object_13 = object.key("metadataColumns").start_object();
        for (key_14, value_15) in var_12 {
            {
                let mut array_16 = object_13.key(key_14.as_str()).start_array();
                for item_17 in value_15 {
                    {
                        array_16.value().string(item_17.as_str());
                    }
                }
                array_16.finish();
            }
        }
        object_13.finish();
    }
    if let Some(var_18) = &input.num_results {
        object.key("numResults").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_18).into()),
        );
    }
    if let Some(var_19) = &input.promotions {
        let mut array_20 = object.key("promotions").start_array();
        for item_21 in var_19 {
            {
                #[allow(unused_mut)]
                let mut object_22 = array_20.value().start_object();
                crate::protocol_serde::shape_promotion::ser_promotion(&mut object_22, item_21)?;
                object_22.finish();
            }
        }
        array_20.finish();
    }
    if let Some(var_23) = &input.recommender_arn {
        object.key("recommenderArn").string(var_23.as_str());
    }
    if let Some(var_24) = &input.user_id {
        object.key("userId").string(var_24.as_str());
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_get_savings_plan_purchase_recommendation_details_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::get_savings_plan_purchase_recommendation_details::GetSavingsPlanPurchaseRecommendationDetailsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.recommendation_detail_id {
        object.key("RecommendationDetailId").string(var_1.as_str());
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_search_products_as_admin_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::search_products_as_admin::SearchProductsAsAdminInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.accept_language {
        object.key("AcceptLanguage").string(var_1.as_str());
    }
    if let Some(var_2) = &input.portfolio_id {
        object.key("PortfolioId").string(var_2.as_str());
    }
    if let Some(var_3) = &input.filters {
        #[allow(unused_mut)]
        let mut object_4 = object.key("Filters").start_object();
        for (key_5, value_6) in var_3 {
            {
                let mut array_7 = object_4.key(key_5.as_str()).start_array();
                for item_8 in value_6 {
                    {
                        array_7.value().string(item_8.as_str());
                    }
                }
                array_7.finish();
            }
        }
        object_4.finish();
    }
    if let Some(var_9) = &input.sort_by {
        object.key("SortBy").string(var_9.as_str());
    }
    if let Some(var_10) = &input.sort_order {
        object.key("SortOrder").string(var_10.as_str());
    }
    if let Some(var_11) = &input.page_token {
        object.key("PageToken").string(var_11.as_str());
    }
    if let Some(var_12) = &input.page_size {
        object.key("PageSize").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_12).into()),
        );
    }
    if let Some(var_13) = &input.product_source {
        object.key("ProductSource").string(var_13.as_str());
    }
    Ok(())
}

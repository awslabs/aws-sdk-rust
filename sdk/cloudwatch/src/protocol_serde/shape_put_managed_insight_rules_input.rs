// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_put_managed_insight_rules_input_input_input(
    input: &crate::operation::put_managed_insight_rules::PutManagedInsightRulesInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "PutManagedInsightRules", "2010-08-01");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("ManagedRules");
    if let Some(var_2) = &input.managed_rules {
        let mut list_4 = scope_1.start_list(false, None);
        for item_3 in var_2 {
            #[allow(unused_mut)]
            let mut entry_5 = list_4.entry();
            crate::protocol_serde::shape_managed_rule::ser_managed_rule(entry_5, item_3)?;
        }
        list_4.finish();
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

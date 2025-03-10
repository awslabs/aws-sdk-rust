// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_get_template_summary_input_input_input(
    input: &crate::operation::get_template_summary::GetTemplateSummaryInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "GetTemplateSummary", "2010-05-15");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("TemplateBody");
    if let Some(var_2) = &input.template_body {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("TemplateURL");
    if let Some(var_4) = &input.template_url {
        scope_3.string(var_4);
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("StackName");
    if let Some(var_6) = &input.stack_name {
        scope_5.string(var_6);
    }
    #[allow(unused_mut)]
    let mut scope_7 = writer.prefix("StackSetName");
    if let Some(var_8) = &input.stack_set_name {
        scope_7.string(var_8);
    }
    #[allow(unused_mut)]
    let mut scope_9 = writer.prefix("CallAs");
    if let Some(var_10) = &input.call_as {
        scope_9.string(var_10.as_str());
    }
    #[allow(unused_mut)]
    let mut scope_11 = writer.prefix("TemplateSummaryConfig");
    if let Some(var_12) = &input.template_summary_config {
        crate::protocol_serde::shape_template_summary_config::ser_template_summary_config(scope_11, var_12)?;
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

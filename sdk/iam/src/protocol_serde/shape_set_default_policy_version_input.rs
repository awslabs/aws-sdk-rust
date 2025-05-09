// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_set_default_policy_version_input_input_input(
    input: &crate::operation::set_default_policy_version::SetDefaultPolicyVersionInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "SetDefaultPolicyVersion", "2010-05-08");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("PolicyArn");
    if let Some(var_2) = &input.policy_arn {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("VersionId");
    if let Some(var_4) = &input.version_id {
        scope_3.string(var_4);
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

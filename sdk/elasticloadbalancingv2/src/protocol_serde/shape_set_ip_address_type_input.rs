// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_set_ip_address_type_input_input_input(
    input: &crate::operation::set_ip_address_type::SetIpAddressTypeInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "SetIpAddressType", "2015-12-01");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("LoadBalancerArn");
    if let Some(var_2) = &input.load_balancer_arn {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("IpAddressType");
    if let Some(var_4) = &input.ip_address_type {
        scope_3.string(var_4.as_str());
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_add_listener_certificates_input_input_input(
    input: &crate::operation::add_listener_certificates::AddListenerCertificatesInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "AddListenerCertificates", "2015-12-01");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("ListenerArn");
    if let Some(var_2) = &input.listener_arn {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("Certificates");
    if let Some(var_4) = &input.certificates {
        let mut list_6 = scope_3.start_list(false, None);
        for item_5 in var_4 {
            #[allow(unused_mut)]
            let mut entry_7 = list_6.entry();
            crate::protocol_serde::shape_certificate::ser_certificate(entry_7, item_5)?;
        }
        list_6.finish();
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

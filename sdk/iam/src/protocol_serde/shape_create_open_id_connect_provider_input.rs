// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_open_id_connect_provider_input_input_input(
    input: &crate::operation::create_open_id_connect_provider::CreateOpenIdConnectProviderInput,
) -> ::std::result::Result<::aws_smithy_types::body::SdkBody, ::aws_smithy_types::error::operation::SerializationError> {
    let mut out = String::new();
    #[allow(unused_mut)]
    let mut writer = ::aws_smithy_query::QueryWriter::new(&mut out, "CreateOpenIDConnectProvider", "2010-05-08");
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("Url");
    if let Some(var_2) = &input.url {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("ClientIDList");
    if let Some(var_4) = &input.client_id_list {
        let mut list_6 = scope_3.start_list(false, None);
        for item_5 in var_4 {
            #[allow(unused_mut)]
            let mut entry_7 = list_6.entry();
            entry_7.string(item_5);
        }
        list_6.finish();
    }
    #[allow(unused_mut)]
    let mut scope_8 = writer.prefix("ThumbprintList");
    if let Some(var_9) = &input.thumbprint_list {
        let mut list_11 = scope_8.start_list(false, None);
        for item_10 in var_9 {
            #[allow(unused_mut)]
            let mut entry_12 = list_11.entry();
            entry_12.string(item_10);
        }
        list_11.finish();
    }
    #[allow(unused_mut)]
    let mut scope_13 = writer.prefix("Tags");
    if let Some(var_14) = &input.tags {
        let mut list_16 = scope_13.start_list(false, None);
        for item_15 in var_14 {
            #[allow(unused_mut)]
            let mut entry_17 = list_16.entry();
            crate::protocol_serde::shape_tag::ser_tag(entry_17, item_15)?;
        }
        list_16.finish();
    }
    writer.finish();
    Ok(::aws_smithy_types::body::SdkBody::from(out))
}

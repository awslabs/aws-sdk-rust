// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(unused_mut)]
pub fn ser_scan_filter(
    mut writer: ::aws_smithy_query::QueryValueWriter,
    input: &crate::types::ScanFilter,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("Types");
    if let Some(var_2) = &input.types {
        let mut list_4 = scope_1.start_list(false, None);
        for item_3 in var_2 {
            #[allow(unused_mut)]
            let mut entry_5 = list_4.entry();
            entry_5.string(item_3);
        }
        list_4.finish();
    }
    Ok(())
}

#[allow(clippy::needless_question_mark)]
pub fn de_scan_filter(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::ScanFilter, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::ScanFilter::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("Types") /* Types com.amazonaws.cloudformation#ScanFilter$Types */ =>  {
                let var_6 =
                    Some(
                        crate::protocol_serde::shape_resource_type_filters::de_resource_type_filters(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_types(var_6);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

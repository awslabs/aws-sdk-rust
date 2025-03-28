// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_event_info_map(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::EventInfoMap, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::EventInfoMap::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("EventId") /* EventId com.amazonaws.redshift#EventInfoMap$EventId */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_event_id(var_1);
            }
            ,
            s if s.matches("EventCategories") /* EventCategories com.amazonaws.redshift#EventInfoMap$EventCategories */ =>  {
                let var_2 =
                    Some(
                        crate::protocol_serde::shape_event_categories_list::de_event_categories_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_event_categories(var_2);
            }
            ,
            s if s.matches("EventDescription") /* EventDescription com.amazonaws.redshift#EventInfoMap$EventDescription */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_event_description(var_3);
            }
            ,
            s if s.matches("Severity") /* Severity com.amazonaws.redshift#EventInfoMap$Severity */ =>  {
                let var_4 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_severity(var_4);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

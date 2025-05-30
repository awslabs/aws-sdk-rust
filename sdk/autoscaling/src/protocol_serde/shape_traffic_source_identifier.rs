// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(unused_mut)]
pub fn ser_traffic_source_identifier(
    mut writer: ::aws_smithy_query::QueryValueWriter,
    input: &crate::types::TrafficSourceIdentifier,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("Identifier");
    if let Some(var_2) = &input.identifier {
        scope_1.string(var_2);
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("Type");
    if let Some(var_4) = &input.r#type {
        scope_3.string(var_4);
    }
    Ok(())
}

#[allow(clippy::needless_question_mark)]
pub fn de_traffic_source_identifier(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::TrafficSourceIdentifier, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::TrafficSourceIdentifier::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("Identifier") /* Identifier com.amazonaws.autoscaling#TrafficSourceIdentifier$Identifier */ =>  {
                let var_5 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_identifier(var_5);
            }
            ,
            s if s.matches("Type") /* Type com.amazonaws.autoscaling#TrafficSourceIdentifier$Type */ =>  {
                let var_6 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_type(var_6);
            }
            ,
            _ => {}
        }
    }
    Ok(crate::serde_util::traffic_source_identifier_correct_errors(builder).build())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_resource_identifier_summary(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::ResourceIdentifierSummary, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::ResourceIdentifierSummary::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("ResourceType") /* ResourceType com.amazonaws.cloudformation#ResourceIdentifierSummary$ResourceType */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_resource_type(var_1);
            }
            ,
            s if s.matches("LogicalResourceIds") /* LogicalResourceIds com.amazonaws.cloudformation#ResourceIdentifierSummary$LogicalResourceIds */ =>  {
                let var_2 =
                    Some(
                        crate::protocol_serde::shape_logical_resource_ids::de_logical_resource_ids(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_logical_resource_ids(var_2);
            }
            ,
            s if s.matches("ResourceIdentifiers") /* ResourceIdentifiers com.amazonaws.cloudformation#ResourceIdentifierSummary$ResourceIdentifiers */ =>  {
                let var_3 =
                    Some(
                        crate::protocol_serde::shape_resource_identifiers::de_resource_identifiers(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_resource_identifiers(var_3);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

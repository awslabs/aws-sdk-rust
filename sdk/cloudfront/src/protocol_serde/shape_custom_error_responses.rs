// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_custom_error_responses(
    input: &crate::types::CustomErrorResponses,
    writer: ::aws_smithy_xml::encode::ElWriter,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope = writer.finish();
    {
        let mut inner_writer = scope.start_el("Quantity").finish();
        inner_writer.data(::aws_smithy_types::primitive::Encoder::from(input.quantity).encode());
    }
    if let Some(var_1) = &input.items {
        let mut inner_writer = scope.start_el("Items").finish();
        for list_item_2 in var_1 {
            {
                let inner_writer = inner_writer.start_el("CustomErrorResponse");
                crate::protocol_serde::shape_custom_error_response::ser_custom_error_response(list_item_2, inner_writer)?
            }
        }
    }
    scope.finish();
    Ok(())
}

#[allow(clippy::needless_question_mark)]
pub fn de_custom_error_responses(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::CustomErrorResponses, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::CustomErrorResponses::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("Quantity") /* Quantity com.amazonaws.cloudfront#CustomErrorResponses$Quantity */ =>  {
                let var_3 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.cloudfront#integer`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_quantity(var_3);
            }
            ,
            s if s.matches("Items") /* Items com.amazonaws.cloudfront#CustomErrorResponses$Items */ =>  {
                let var_4 =
                    Some(
                        crate::protocol_serde::shape_custom_error_response_list::de_custom_error_response_list(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_items(var_4);
            }
            ,
            _ => {}
        }
    }
    Ok(crate::serde_util::custom_error_responses_correct_errors(builder)
        .build()
        .map_err(|_| ::aws_smithy_xml::decode::XmlDecodeError::custom("missing field"))?)
}

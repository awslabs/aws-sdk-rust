// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_delete_multi_region_access_point_input_input_input(
    input: &crate::operation::delete_multi_region_access_point::DeleteMultiRegionAccessPointInput,
    writer: ::aws_smithy_xml::encode::ElWriter,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope = writer.finish();
    if let Some(var_1) = &input.client_token {
        let mut inner_writer = scope.start_el("ClientToken").finish();
        inner_writer.data(var_1.as_str());
    }
    if let Some(var_2) = &input.details {
        let inner_writer = scope.start_el("Details");
        crate::protocol_serde::shape_delete_multi_region_access_point_input::ser_delete_multi_region_access_point_input(var_2, inner_writer)?
    }
    scope.finish();
    Ok(())
}

pub fn ser_delete_multi_region_access_point_input(
    input: &crate::types::DeleteMultiRegionAccessPointInput,
    writer: ::aws_smithy_xml::encode::ElWriter,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope = writer.finish();
    {
        let mut inner_writer = scope.start_el("Name").finish();
        inner_writer.data(input.name.as_str());
    }
    scope.finish();
    Ok(())
}

#[allow(clippy::needless_question_mark)]
pub fn de_delete_multi_region_access_point_input(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::DeleteMultiRegionAccessPointInput, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::DeleteMultiRegionAccessPointInput::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("Name") /* Name com.amazonaws.s3control#DeleteMultiRegionAccessPointInput$Name */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_name(var_3);
            }
            ,
            _ => {}
        }
    }
    Ok(crate::serde_util::delete_multi_region_access_point_input_correct_errors(builder)
        .build()
        .map_err(|_| ::aws_smithy_xml::decode::XmlDecodeError::custom("missing field"))?)
}

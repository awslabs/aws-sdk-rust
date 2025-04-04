// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_notification_configuration_filter(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::NotificationConfigurationFilter, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::NotificationConfigurationFilter::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("S3Key") /* Key com.amazonaws.s3#NotificationConfigurationFilter$Key */ =>  {
                let var_1 =
                    Some(
                        crate::protocol_serde::shape_s3_key_filter::de_s3_key_filter(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_key(var_1);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

pub fn ser_notification_configuration_filter(
    input: &crate::types::NotificationConfigurationFilter,
    writer: ::aws_smithy_xml::encode::ElWriter,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope = writer.finish();
    if let Some(var_2) = &input.key {
        let inner_writer = scope.start_el("S3Key");
        crate::protocol_serde::shape_s3_key_filter::ser_s3_key_filter(var_2, inner_writer)?
    }
    scope.finish();
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_field_level_encryption_config(
    input: &crate::types::FieldLevelEncryptionConfig,
    writer: ::aws_smithy_xml::encode::ElWriter,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope = writer.finish();
    {
        let mut inner_writer = scope.start_el("CallerReference").finish();
        inner_writer.data(input.caller_reference.as_str());
    }
    if let Some(var_1) = &input.comment {
        let mut inner_writer = scope.start_el("Comment").finish();
        inner_writer.data(var_1.as_str());
    }
    if let Some(var_2) = &input.query_arg_profile_config {
        let inner_writer = scope.start_el("QueryArgProfileConfig");
        crate::protocol_serde::shape_query_arg_profile_config::ser_query_arg_profile_config(var_2, inner_writer)?
    }
    if let Some(var_3) = &input.content_type_profile_config {
        let inner_writer = scope.start_el("ContentTypeProfileConfig");
        crate::protocol_serde::shape_content_type_profile_config::ser_content_type_profile_config(var_3, inner_writer)?
    }
    scope.finish();
    Ok(())
}

#[allow(clippy::needless_question_mark)]
pub fn de_field_level_encryption_config(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::FieldLevelEncryptionConfig, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::FieldLevelEncryptionConfig::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("CallerReference") /* CallerReference com.amazonaws.cloudfront#FieldLevelEncryptionConfig$CallerReference */ =>  {
                let var_4 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_caller_reference(var_4);
            }
            ,
            s if s.matches("Comment") /* Comment com.amazonaws.cloudfront#FieldLevelEncryptionConfig$Comment */ =>  {
                let var_5 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_comment(var_5);
            }
            ,
            s if s.matches("QueryArgProfileConfig") /* QueryArgProfileConfig com.amazonaws.cloudfront#FieldLevelEncryptionConfig$QueryArgProfileConfig */ =>  {
                let var_6 =
                    Some(
                        crate::protocol_serde::shape_query_arg_profile_config::de_query_arg_profile_config(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_query_arg_profile_config(var_6);
            }
            ,
            s if s.matches("ContentTypeProfileConfig") /* ContentTypeProfileConfig com.amazonaws.cloudfront#FieldLevelEncryptionConfig$ContentTypeProfileConfig */ =>  {
                let var_7 =
                    Some(
                        crate::protocol_serde::shape_content_type_profile_config::de_content_type_profile_config(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_content_type_profile_config(var_7);
            }
            ,
            _ => {}
        }
    }
    Ok(crate::serde_util::field_level_encryption_config_correct_errors(builder)
        .build()
        .map_err(|_| ::aws_smithy_xml::decode::XmlDecodeError::custom("missing field"))?)
}

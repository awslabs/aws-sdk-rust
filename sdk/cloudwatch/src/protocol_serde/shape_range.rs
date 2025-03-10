// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(unused_mut)]
pub fn ser_range(
    mut writer: ::aws_smithy_query::QueryValueWriter,
    input: &crate::types::Range,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("StartTime");
    if let Some(var_2) = &input.start_time {
        scope_1.date_time(var_2, ::aws_smithy_types::date_time::Format::DateTime)?;
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("EndTime");
    if let Some(var_4) = &input.end_time {
        scope_3.date_time(var_4, ::aws_smithy_types::date_time::Format::DateTime)?;
    }
    Ok(())
}

#[allow(clippy::needless_question_mark)]
pub fn de_range(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::Range, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::Range::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("StartTime") /* StartTime com.amazonaws.cloudwatch#Range$StartTime */ =>  {
                let var_5 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.cloudwatch#Timestamp`)"))
                        ?
                    )
                ;
                builder = builder.set_start_time(var_5);
            }
            ,
            s if s.matches("EndTime") /* EndTime com.amazonaws.cloudwatch#Range$EndTime */ =>  {
                let var_6 =
                    Some(
                        ::aws_smithy_types::DateTime::from_str(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            , ::aws_smithy_types::date_time::Format::DateTimeWithOffset
                        )
                        .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (timestamp: `com.amazonaws.cloudwatch#Timestamp`)"))
                        ?
                    )
                ;
                builder = builder.set_end_time(var_6);
            }
            ,
            _ => {}
        }
    }
    Ok(crate::serde_util::range_correct_errors(builder).build())
}

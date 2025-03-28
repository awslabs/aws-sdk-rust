// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_limitless_database(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::LimitlessDatabase, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::LimitlessDatabase::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("Status") /* Status com.amazonaws.rds#LimitlessDatabase$Status */ =>  {
                let var_1 =
                    Some(
                        Result::<crate::types::LimitlessDatabaseStatus, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::LimitlessDatabaseStatus::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_status(var_1);
            }
            ,
            s if s.matches("MinRequiredACU") /* MinRequiredACU com.amazonaws.rds#LimitlessDatabase$MinRequiredACU */ =>  {
                let var_2 =
                    Some(
                         {
                            <f64 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (double: `com.amazonaws.rds#DoubleOptional`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_min_required_acu(var_2);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_lifecycle_rule_filter(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::LifecycleRuleFilter, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::LifecycleRuleFilter::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("Prefix") /* Prefix com.amazonaws.s3#LifecycleRuleFilter$Prefix */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_prefix(var_1);
            }
            ,
            s if s.matches("Tag") /* Tag com.amazonaws.s3#LifecycleRuleFilter$Tag */ =>  {
                let var_2 =
                    Some(
                        crate::protocol_serde::shape_tag::de_tag(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_tag(var_2);
            }
            ,
            s if s.matches("ObjectSizeGreaterThan") /* ObjectSizeGreaterThan com.amazonaws.s3#LifecycleRuleFilter$ObjectSizeGreaterThan */ =>  {
                let var_3 =
                    Some(
                         {
                            <i64 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (long: `com.amazonaws.s3#ObjectSizeGreaterThanBytes`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_object_size_greater_than(var_3);
            }
            ,
            s if s.matches("ObjectSizeLessThan") /* ObjectSizeLessThan com.amazonaws.s3#LifecycleRuleFilter$ObjectSizeLessThan */ =>  {
                let var_4 =
                    Some(
                         {
                            <i64 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (long: `com.amazonaws.s3#ObjectSizeLessThanBytes`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_object_size_less_than(var_4);
            }
            ,
            s if s.matches("And") /* And com.amazonaws.s3#LifecycleRuleFilter$And */ =>  {
                let var_5 =
                    Some(
                        crate::protocol_serde::shape_lifecycle_rule_and_operator::de_lifecycle_rule_and_operator(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_and(var_5);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

pub fn ser_lifecycle_rule_filter(
    input: &crate::types::LifecycleRuleFilter,
    writer: ::aws_smithy_xml::encode::ElWriter,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope = writer.finish();
    if let Some(var_6) = &input.prefix {
        let mut inner_writer = scope.start_el("Prefix").finish();
        inner_writer.data(var_6.as_str());
    }
    if let Some(var_7) = &input.tag {
        let inner_writer = scope.start_el("Tag");
        crate::protocol_serde::shape_tag::ser_tag(var_7, inner_writer)?
    }
    if let Some(var_8) = &input.object_size_greater_than {
        let mut inner_writer = scope.start_el("ObjectSizeGreaterThan").finish();
        inner_writer.data(::aws_smithy_types::primitive::Encoder::from(*var_8).encode());
    }
    if let Some(var_9) = &input.object_size_less_than {
        let mut inner_writer = scope.start_el("ObjectSizeLessThan").finish();
        inner_writer.data(::aws_smithy_types::primitive::Encoder::from(*var_9).encode());
    }
    if let Some(var_10) = &input.and {
        let inner_writer = scope.start_el("And");
        crate::protocol_serde::shape_lifecycle_rule_and_operator::ser_lifecycle_rule_and_operator(var_10, inner_writer)?
    }
    scope.finish();
    Ok(())
}

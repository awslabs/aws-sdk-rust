// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(unused_mut)]
pub fn ser_int_options(
    mut writer: ::aws_smithy_query::QueryValueWriter,
    input: &crate::types::IntOptions,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    #[allow(unused_mut)]
    let mut scope_1 = writer.prefix("DefaultValue");
    if let Some(var_2) = &input.default_value {
        scope_1.number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_2).into()),
        );
    }
    #[allow(unused_mut)]
    let mut scope_3 = writer.prefix("SourceField");
    if let Some(var_4) = &input.source_field {
        scope_3.string(var_4);
    }
    #[allow(unused_mut)]
    let mut scope_5 = writer.prefix("FacetEnabled");
    if let Some(var_6) = &input.facet_enabled {
        scope_5.boolean(*var_6);
    }
    #[allow(unused_mut)]
    let mut scope_7 = writer.prefix("SearchEnabled");
    if let Some(var_8) = &input.search_enabled {
        scope_7.boolean(*var_8);
    }
    #[allow(unused_mut)]
    let mut scope_9 = writer.prefix("ReturnEnabled");
    if let Some(var_10) = &input.return_enabled {
        scope_9.boolean(*var_10);
    }
    #[allow(unused_mut)]
    let mut scope_11 = writer.prefix("SortEnabled");
    if let Some(var_12) = &input.sort_enabled {
        scope_11.boolean(*var_12);
    }
    Ok(())
}

#[allow(clippy::needless_question_mark)]
pub fn de_int_options(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::IntOptions, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::IntOptions::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("DefaultValue") /* DefaultValue com.amazonaws.cloudsearch#IntOptions$DefaultValue */ =>  {
                let var_13 =
                    Some(
                         {
                            <i64 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (long: `com.amazonaws.cloudsearch#Long`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_default_value(var_13);
            }
            ,
            s if s.matches("SourceField") /* SourceField com.amazonaws.cloudsearch#IntOptions$SourceField */ =>  {
                let var_14 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_source_field(var_14);
            }
            ,
            s if s.matches("FacetEnabled") /* FacetEnabled com.amazonaws.cloudsearch#IntOptions$FacetEnabled */ =>  {
                let var_15 =
                    Some(
                         {
                            <bool as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (boolean: `com.amazonaws.cloudsearch#Boolean`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_facet_enabled(var_15);
            }
            ,
            s if s.matches("SearchEnabled") /* SearchEnabled com.amazonaws.cloudsearch#IntOptions$SearchEnabled */ =>  {
                let var_16 =
                    Some(
                         {
                            <bool as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (boolean: `com.amazonaws.cloudsearch#Boolean`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_search_enabled(var_16);
            }
            ,
            s if s.matches("ReturnEnabled") /* ReturnEnabled com.amazonaws.cloudsearch#IntOptions$ReturnEnabled */ =>  {
                let var_17 =
                    Some(
                         {
                            <bool as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (boolean: `com.amazonaws.cloudsearch#Boolean`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_return_enabled(var_17);
            }
            ,
            s if s.matches("SortEnabled") /* SortEnabled com.amazonaws.cloudsearch#IntOptions$SortEnabled */ =>  {
                let var_18 =
                    Some(
                         {
                            <bool as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (boolean: `com.amazonaws.cloudsearch#Boolean`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_sort_enabled(var_18);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

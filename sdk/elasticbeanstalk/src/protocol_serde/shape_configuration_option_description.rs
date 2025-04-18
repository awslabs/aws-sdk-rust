// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
#[allow(clippy::needless_question_mark)]
pub fn de_configuration_option_description(
    decoder: &mut ::aws_smithy_xml::decode::ScopedDecoder,
) -> ::std::result::Result<crate::types::ConfigurationOptionDescription, ::aws_smithy_xml::decode::XmlDecodeError> {
    #[allow(unused_mut)]
    let mut builder = crate::types::ConfigurationOptionDescription::builder();
    while let Some(mut tag) = decoder.next_tag() {
        match tag.start_el() {
            s if s.matches("Namespace") /* Namespace com.amazonaws.elasticbeanstalk#ConfigurationOptionDescription$Namespace */ =>  {
                let var_1 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_namespace(var_1);
            }
            ,
            s if s.matches("Name") /* Name com.amazonaws.elasticbeanstalk#ConfigurationOptionDescription$Name */ =>  {
                let var_2 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_name(var_2);
            }
            ,
            s if s.matches("DefaultValue") /* DefaultValue com.amazonaws.elasticbeanstalk#ConfigurationOptionDescription$DefaultValue */ =>  {
                let var_3 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_default_value(var_3);
            }
            ,
            s if s.matches("ChangeSeverity") /* ChangeSeverity com.amazonaws.elasticbeanstalk#ConfigurationOptionDescription$ChangeSeverity */ =>  {
                let var_4 =
                    Some(
                        Result::<::std::string::String, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            .into()
                        )
                        ?
                    )
                ;
                builder = builder.set_change_severity(var_4);
            }
            ,
            s if s.matches("UserDefined") /* UserDefined com.amazonaws.elasticbeanstalk#ConfigurationOptionDescription$UserDefined */ =>  {
                let var_5 =
                    Some(
                         {
                            <bool as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (boolean: `com.amazonaws.elasticbeanstalk#UserDefinedOption`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_user_defined(var_5);
            }
            ,
            s if s.matches("ValueType") /* ValueType com.amazonaws.elasticbeanstalk#ConfigurationOptionDescription$ValueType */ =>  {
                let var_6 =
                    Some(
                        Result::<crate::types::ConfigurationOptionValueType, ::aws_smithy_xml::decode::XmlDecodeError>::Ok(
                            crate::types::ConfigurationOptionValueType::from(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                        )
                        ?
                    )
                ;
                builder = builder.set_value_type(var_6);
            }
            ,
            s if s.matches("ValueOptions") /* ValueOptions com.amazonaws.elasticbeanstalk#ConfigurationOptionDescription$ValueOptions */ =>  {
                let var_7 =
                    Some(
                        crate::protocol_serde::shape_configuration_option_possible_values::de_configuration_option_possible_values(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_value_options(var_7);
            }
            ,
            s if s.matches("MinValue") /* MinValue com.amazonaws.elasticbeanstalk#ConfigurationOptionDescription$MinValue */ =>  {
                let var_8 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.elasticbeanstalk#OptionRestrictionMinValue`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_min_value(var_8);
            }
            ,
            s if s.matches("MaxValue") /* MaxValue com.amazonaws.elasticbeanstalk#ConfigurationOptionDescription$MaxValue */ =>  {
                let var_9 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.elasticbeanstalk#OptionRestrictionMaxValue`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_max_value(var_9);
            }
            ,
            s if s.matches("MaxLength") /* MaxLength com.amazonaws.elasticbeanstalk#ConfigurationOptionDescription$MaxLength */ =>  {
                let var_10 =
                    Some(
                         {
                            <i32 as ::aws_smithy_types::primitive::Parse>::parse_smithy_primitive(
                                ::aws_smithy_xml::decode::try_data(&mut tag)?.as_ref()
                            )
                            .map_err(|_|::aws_smithy_xml::decode::XmlDecodeError::custom("expected (integer: `com.amazonaws.elasticbeanstalk#OptionRestrictionMaxLength`)"))
                        }
                        ?
                    )
                ;
                builder = builder.set_max_length(var_10);
            }
            ,
            s if s.matches("Regex") /* Regex com.amazonaws.elasticbeanstalk#ConfigurationOptionDescription$Regex */ =>  {
                let var_11 =
                    Some(
                        crate::protocol_serde::shape_option_restriction_regex::de_option_restriction_regex(&mut tag)
                        ?
                    )
                ;
                builder = builder.set_regex(var_11);
            }
            ,
            _ => {}
        }
    }
    Ok(builder.build())
}

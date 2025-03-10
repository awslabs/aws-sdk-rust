// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_storage_descriptor<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::StorageDescriptor>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::StorageDescriptorBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "Columns" => {
                            builder = builder.set_columns(crate::protocol_serde::shape_column_list::de_column_list(tokens)?);
                        }
                        "Location" => {
                            builder = builder.set_location(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "AdditionalLocations" => {
                            builder =
                                builder.set_additional_locations(crate::protocol_serde::shape_location_string_list::de_location_string_list(tokens)?);
                        }
                        "InputFormat" => {
                            builder = builder.set_input_format(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "OutputFormat" => {
                            builder = builder.set_output_format(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "Compressed" => {
                            builder = builder.set_compressed(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "NumberOfBuckets" => {
                            builder = builder.set_number_of_buckets(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "SerdeInfo" => {
                            builder = builder.set_serde_info(crate::protocol_serde::shape_ser_de_info::de_ser_de_info(tokens)?);
                        }
                        "BucketColumns" => {
                            builder = builder.set_bucket_columns(crate::protocol_serde::shape_name_string_list::de_name_string_list(tokens)?);
                        }
                        "SortColumns" => {
                            builder = builder.set_sort_columns(crate::protocol_serde::shape_order_list::de_order_list(tokens)?);
                        }
                        "Parameters" => {
                            builder = builder.set_parameters(crate::protocol_serde::shape_parameters_map::de_parameters_map(tokens)?);
                        }
                        "SkewedInfo" => {
                            builder = builder.set_skewed_info(crate::protocol_serde::shape_skewed_info::de_skewed_info(tokens)?);
                        }
                        "StoredAsSubDirectories" => {
                            builder =
                                builder.set_stored_as_sub_directories(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "SchemaReference" => {
                            builder = builder.set_schema_reference(crate::protocol_serde::shape_schema_reference::de_schema_reference(tokens)?);
                        }
                        _ => ::aws_smithy_json::deserialize::token::skip_value(tokens)?,
                    },
                    other => {
                        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(format!(
                            "expected object key or end object, found: {:?}",
                            other
                        )))
                    }
                }
            }
            Ok(Some(builder.build()))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

pub fn ser_storage_descriptor(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::StorageDescriptor,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.columns {
        let mut array_2 = object.key("Columns").start_array();
        for item_3 in var_1 {
            {
                #[allow(unused_mut)]
                let mut object_4 = array_2.value().start_object();
                crate::protocol_serde::shape_column::ser_column(&mut object_4, item_3)?;
                object_4.finish();
            }
        }
        array_2.finish();
    }
    if let Some(var_5) = &input.location {
        object.key("Location").string(var_5.as_str());
    }
    if let Some(var_6) = &input.additional_locations {
        let mut array_7 = object.key("AdditionalLocations").start_array();
        for item_8 in var_6 {
            {
                array_7.value().string(item_8.as_str());
            }
        }
        array_7.finish();
    }
    if let Some(var_9) = &input.input_format {
        object.key("InputFormat").string(var_9.as_str());
    }
    if let Some(var_10) = &input.output_format {
        object.key("OutputFormat").string(var_10.as_str());
    }
    if input.compressed {
        object.key("Compressed").boolean(input.compressed);
    }
    if input.number_of_buckets != 0 {
        object.key("NumberOfBuckets").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.number_of_buckets).into()),
        );
    }
    if let Some(var_11) = &input.serde_info {
        #[allow(unused_mut)]
        let mut object_12 = object.key("SerdeInfo").start_object();
        crate::protocol_serde::shape_ser_de_info::ser_ser_de_info(&mut object_12, var_11)?;
        object_12.finish();
    }
    if let Some(var_13) = &input.bucket_columns {
        let mut array_14 = object.key("BucketColumns").start_array();
        for item_15 in var_13 {
            {
                array_14.value().string(item_15.as_str());
            }
        }
        array_14.finish();
    }
    if let Some(var_16) = &input.sort_columns {
        let mut array_17 = object.key("SortColumns").start_array();
        for item_18 in var_16 {
            {
                #[allow(unused_mut)]
                let mut object_19 = array_17.value().start_object();
                crate::protocol_serde::shape_order::ser_order(&mut object_19, item_18)?;
                object_19.finish();
            }
        }
        array_17.finish();
    }
    if let Some(var_20) = &input.parameters {
        #[allow(unused_mut)]
        let mut object_21 = object.key("Parameters").start_object();
        for (key_22, value_23) in var_20 {
            {
                object_21.key(key_22.as_str()).string(value_23.as_str());
            }
        }
        object_21.finish();
    }
    if let Some(var_24) = &input.skewed_info {
        #[allow(unused_mut)]
        let mut object_25 = object.key("SkewedInfo").start_object();
        crate::protocol_serde::shape_skewed_info::ser_skewed_info(&mut object_25, var_24)?;
        object_25.finish();
    }
    if input.stored_as_sub_directories {
        object.key("StoredAsSubDirectories").boolean(input.stored_as_sub_directories);
    }
    if let Some(var_26) = &input.schema_reference {
        #[allow(unused_mut)]
        let mut object_27 = object.key("SchemaReference").start_object();
        crate::protocol_serde::shape_schema_reference::ser_schema_reference(&mut object_27, var_26)?;
        object_27.finish();
    }
    Ok(())
}

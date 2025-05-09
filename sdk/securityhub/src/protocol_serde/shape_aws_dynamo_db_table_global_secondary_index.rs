// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_aws_dynamo_db_table_global_secondary_index(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::AwsDynamoDbTableGlobalSecondaryIndex,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.backfilling {
        object.key("Backfilling").boolean(*var_1);
    }
    if let Some(var_2) = &input.index_arn {
        object.key("IndexArn").string(var_2.as_str());
    }
    if let Some(var_3) = &input.index_name {
        object.key("IndexName").string(var_3.as_str());
    }
    if let Some(var_4) = &input.index_size_bytes {
        object.key("IndexSizeBytes").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_4).into()),
        );
    }
    if let Some(var_5) = &input.index_status {
        object.key("IndexStatus").string(var_5.as_str());
    }
    if let Some(var_6) = &input.item_count {
        object.key("ItemCount").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_6).into()),
        );
    }
    if let Some(var_7) = &input.key_schema {
        let mut array_8 = object.key("KeySchema").start_array();
        for item_9 in var_7 {
            {
                #[allow(unused_mut)]
                let mut object_10 = array_8.value().start_object();
                crate::protocol_serde::shape_aws_dynamo_db_table_key_schema::ser_aws_dynamo_db_table_key_schema(&mut object_10, item_9)?;
                object_10.finish();
            }
        }
        array_8.finish();
    }
    if let Some(var_11) = &input.projection {
        #[allow(unused_mut)]
        let mut object_12 = object.key("Projection").start_object();
        crate::protocol_serde::shape_aws_dynamo_db_table_projection::ser_aws_dynamo_db_table_projection(&mut object_12, var_11)?;
        object_12.finish();
    }
    if let Some(var_13) = &input.provisioned_throughput {
        #[allow(unused_mut)]
        let mut object_14 = object.key("ProvisionedThroughput").start_object();
        crate::protocol_serde::shape_aws_dynamo_db_table_provisioned_throughput::ser_aws_dynamo_db_table_provisioned_throughput(
            &mut object_14,
            var_13,
        )?;
        object_14.finish();
    }
    Ok(())
}

pub(crate) fn de_aws_dynamo_db_table_global_secondary_index<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::AwsDynamoDbTableGlobalSecondaryIndex>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::AwsDynamoDbTableGlobalSecondaryIndexBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "Backfilling" => {
                            builder = builder.set_backfilling(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "IndexArn" => {
                            builder = builder.set_index_arn(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "IndexName" => {
                            builder = builder.set_index_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "IndexSizeBytes" => {
                            builder = builder.set_index_size_bytes(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i64::try_from)
                                    .transpose()?,
                            );
                        }
                        "IndexStatus" => {
                            builder = builder.set_index_status(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "ItemCount" => {
                            builder = builder.set_item_count(
                                ::aws_smithy_json::deserialize::token::expect_number_or_null(tokens.next())?
                                    .map(i32::try_from)
                                    .transpose()?,
                            );
                        }
                        "KeySchema" => {
                            builder = builder.set_key_schema(
                                crate::protocol_serde::shape_aws_dynamo_db_table_key_schema_list::de_aws_dynamo_db_table_key_schema_list(tokens)?,
                            );
                        }
                        "Projection" => {
                            builder = builder.set_projection(
                                crate::protocol_serde::shape_aws_dynamo_db_table_projection::de_aws_dynamo_db_table_projection(tokens)?,
                            );
                        }
                        "ProvisionedThroughput" => {
                            builder = builder.set_provisioned_throughput(
                                    crate::protocol_serde::shape_aws_dynamo_db_table_provisioned_throughput::de_aws_dynamo_db_table_provisioned_throughput(tokens)?
                                );
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

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_datastore_iot_site_wise_multi_layer_storage(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::DatastoreIotSiteWiseMultiLayerStorage,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.customer_managed_s3_storage {
        #[allow(unused_mut)]
        let mut object_2 = object.key("customerManagedS3Storage").start_object();
        crate::protocol_serde::shape_iot_site_wise_customer_managed_datastore_s3_storage::ser_iot_site_wise_customer_managed_datastore_s3_storage(
            &mut object_2,
            var_1,
        )?;
        object_2.finish();
    }
    Ok(())
}

pub(crate) fn de_datastore_iot_site_wise_multi_layer_storage<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::DatastoreIotSiteWiseMultiLayerStorage>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::DatastoreIotSiteWiseMultiLayerStorageBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "customerManagedS3Storage" => {
                            builder = builder.set_customer_managed_s3_storage(
                                    crate::protocol_serde::shape_iot_site_wise_customer_managed_datastore_s3_storage::de_iot_site_wise_customer_managed_datastore_s3_storage(tokens)?
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
            Ok(Some(
                crate::serde_util::datastore_iot_site_wise_multi_layer_storage_correct_errors(builder).build(),
            ))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

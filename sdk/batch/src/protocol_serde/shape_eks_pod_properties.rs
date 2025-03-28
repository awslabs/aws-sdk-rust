// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_eks_pod_properties(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::EksPodProperties,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.service_account_name {
        object.key("serviceAccountName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.host_network {
        object.key("hostNetwork").boolean(*var_2);
    }
    if let Some(var_3) = &input.dns_policy {
        object.key("dnsPolicy").string(var_3.as_str());
    }
    if let Some(var_4) = &input.image_pull_secrets {
        let mut array_5 = object.key("imagePullSecrets").start_array();
        for item_6 in var_4 {
            {
                #[allow(unused_mut)]
                let mut object_7 = array_5.value().start_object();
                crate::protocol_serde::shape_image_pull_secret::ser_image_pull_secret(&mut object_7, item_6)?;
                object_7.finish();
            }
        }
        array_5.finish();
    }
    if let Some(var_8) = &input.containers {
        let mut array_9 = object.key("containers").start_array();
        for item_10 in var_8 {
            {
                #[allow(unused_mut)]
                let mut object_11 = array_9.value().start_object();
                crate::protocol_serde::shape_eks_container::ser_eks_container(&mut object_11, item_10)?;
                object_11.finish();
            }
        }
        array_9.finish();
    }
    if let Some(var_12) = &input.init_containers {
        let mut array_13 = object.key("initContainers").start_array();
        for item_14 in var_12 {
            {
                #[allow(unused_mut)]
                let mut object_15 = array_13.value().start_object();
                crate::protocol_serde::shape_eks_container::ser_eks_container(&mut object_15, item_14)?;
                object_15.finish();
            }
        }
        array_13.finish();
    }
    if let Some(var_16) = &input.volumes {
        let mut array_17 = object.key("volumes").start_array();
        for item_18 in var_16 {
            {
                #[allow(unused_mut)]
                let mut object_19 = array_17.value().start_object();
                crate::protocol_serde::shape_eks_volume::ser_eks_volume(&mut object_19, item_18)?;
                object_19.finish();
            }
        }
        array_17.finish();
    }
    if let Some(var_20) = &input.metadata {
        #[allow(unused_mut)]
        let mut object_21 = object.key("metadata").start_object();
        crate::protocol_serde::shape_eks_metadata::ser_eks_metadata(&mut object_21, var_20)?;
        object_21.finish();
    }
    if let Some(var_22) = &input.share_process_namespace {
        object.key("shareProcessNamespace").boolean(*var_22);
    }
    Ok(())
}

pub(crate) fn de_eks_pod_properties<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::EksPodProperties>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::EksPodPropertiesBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "serviceAccountName" => {
                            builder = builder.set_service_account_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "hostNetwork" => {
                            builder = builder.set_host_network(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "dnsPolicy" => {
                            builder = builder.set_dns_policy(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "imagePullSecrets" => {
                            builder = builder.set_image_pull_secrets(crate::protocol_serde::shape_image_pull_secrets::de_image_pull_secrets(tokens)?);
                        }
                        "containers" => {
                            builder = builder.set_containers(crate::protocol_serde::shape_eks_containers::de_eks_containers(tokens)?);
                        }
                        "initContainers" => {
                            builder = builder.set_init_containers(crate::protocol_serde::shape_eks_containers::de_eks_containers(tokens)?);
                        }
                        "volumes" => {
                            builder = builder.set_volumes(crate::protocol_serde::shape_eks_volumes::de_eks_volumes(tokens)?);
                        }
                        "metadata" => {
                            builder = builder.set_metadata(crate::protocol_serde::shape_eks_metadata::de_eks_metadata(tokens)?);
                        }
                        "shareProcessNamespace" => {
                            builder = builder.set_share_process_namespace(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
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

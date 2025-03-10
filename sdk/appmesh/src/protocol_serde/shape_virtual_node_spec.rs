// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_virtual_node_spec(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::VirtualNodeSpec,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.service_discovery {
        #[allow(unused_mut)]
        let mut object_2 = object.key("serviceDiscovery").start_object();
        crate::protocol_serde::shape_service_discovery::ser_service_discovery(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.listeners {
        let mut array_4 = object.key("listeners").start_array();
        for item_5 in var_3 {
            {
                #[allow(unused_mut)]
                let mut object_6 = array_4.value().start_object();
                crate::protocol_serde::shape_listener::ser_listener(&mut object_6, item_5)?;
                object_6.finish();
            }
        }
        array_4.finish();
    }
    if let Some(var_7) = &input.backends {
        let mut array_8 = object.key("backends").start_array();
        for item_9 in var_7 {
            {
                #[allow(unused_mut)]
                let mut object_10 = array_8.value().start_object();
                crate::protocol_serde::shape_backend::ser_backend(&mut object_10, item_9)?;
                object_10.finish();
            }
        }
        array_8.finish();
    }
    if let Some(var_11) = &input.backend_defaults {
        #[allow(unused_mut)]
        let mut object_12 = object.key("backendDefaults").start_object();
        crate::protocol_serde::shape_backend_defaults::ser_backend_defaults(&mut object_12, var_11)?;
        object_12.finish();
    }
    if let Some(var_13) = &input.logging {
        #[allow(unused_mut)]
        let mut object_14 = object.key("logging").start_object();
        crate::protocol_serde::shape_logging::ser_logging(&mut object_14, var_13)?;
        object_14.finish();
    }
    Ok(())
}

pub(crate) fn de_virtual_node_spec<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::VirtualNodeSpec>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::VirtualNodeSpecBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "serviceDiscovery" => {
                            builder = builder.set_service_discovery(crate::protocol_serde::shape_service_discovery::de_service_discovery(tokens)?);
                        }
                        "listeners" => {
                            builder = builder.set_listeners(crate::protocol_serde::shape_listeners::de_listeners(tokens)?);
                        }
                        "backends" => {
                            builder = builder.set_backends(crate::protocol_serde::shape_backends::de_backends(tokens)?);
                        }
                        "backendDefaults" => {
                            builder = builder.set_backend_defaults(crate::protocol_serde::shape_backend_defaults::de_backend_defaults(tokens)?);
                        }
                        "logging" => {
                            builder = builder.set_logging(crate::protocol_serde::shape_logging::de_logging(tokens)?);
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

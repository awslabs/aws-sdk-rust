// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn de_event_notification_item_configurations<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::EventNotificationItemConfigurations>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::EventNotificationItemConfigurationsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "DeviceRegistrationState" => {
                            builder = builder.set_device_registration_state(
                                    crate::protocol_serde::shape_device_registration_state_event_configuration::de_device_registration_state_event_configuration(tokens)?
                                );
                        }
                        "Proximity" => {
                            builder = builder
                                .set_proximity(crate::protocol_serde::shape_proximity_event_configuration::de_proximity_event_configuration(tokens)?);
                        }
                        "Join" => {
                            builder = builder.set_join(crate::protocol_serde::shape_join_event_configuration::de_join_event_configuration(
                                tokens,
                            )?);
                        }
                        "ConnectionStatus" => {
                            builder = builder.set_connection_status(
                                crate::protocol_serde::shape_connection_status_event_configuration::de_connection_status_event_configuration(tokens)?,
                            );
                        }
                        "MessageDeliveryStatus" => {
                            builder = builder.set_message_delivery_status(
                                    crate::protocol_serde::shape_message_delivery_status_event_configuration::de_message_delivery_status_event_configuration(tokens)?
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

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_configuration_event(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ConfigurationEvent,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.request_attributes {
        #[allow(unused_mut)]
        let mut object_2 = object.key("requestAttributes").start_object();
        for (key_3, value_4) in var_1 {
            {
                object_2.key(key_3.as_str()).string(value_4.as_str());
            }
        }
        object_2.finish();
    }
    {
        object.key("responseContentType").string(input.response_content_type.as_str());
    }
    if let Some(var_5) = &input.session_state {
        #[allow(unused_mut)]
        let mut object_6 = object.key("sessionState").start_object();
        crate::protocol_serde::shape_session_state::ser_session_state(&mut object_6, var_5)?;
        object_6.finish();
    }
    if let Some(var_7) = &input.welcome_messages {
        let mut array_8 = object.key("welcomeMessages").start_array();
        for item_9 in var_7 {
            {
                #[allow(unused_mut)]
                let mut object_10 = array_8.value().start_object();
                crate::protocol_serde::shape_message::ser_message(&mut object_10, item_9)?;
                object_10.finish();
            }
        }
        array_8.finish();
    }
    if input.disable_playback {
        object.key("disablePlayback").boolean(input.disable_playback);
    }
    if let Some(var_11) = &input.event_id {
        object.key("eventId").string(var_11.as_str());
    }
    if input.client_timestamp_millis != 0 {
        object.key("clientTimestampMillis").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.client_timestamp_millis).into()),
        );
    }
    Ok(())
}

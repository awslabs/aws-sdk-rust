// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_subscriber_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_subscriber::UpdateSubscriberInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.sources {
        let mut array_2 = object.key("sources").start_array();
        for item_3 in var_1 {
            {
                #[allow(unused_mut)]
                let mut object_4 = array_2.value().start_object();
                crate::protocol_serde::shape_log_source_resource::ser_log_source_resource(&mut object_4, item_3)?;
                object_4.finish();
            }
        }
        array_2.finish();
    }
    if let Some(var_5) = &input.subscriber_description {
        object.key("subscriberDescription").string(var_5.as_str());
    }
    if let Some(var_6) = &input.subscriber_identity {
        #[allow(unused_mut)]
        let mut object_7 = object.key("subscriberIdentity").start_object();
        crate::protocol_serde::shape_aws_identity::ser_aws_identity(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.subscriber_name {
        object.key("subscriberName").string(var_8.as_str());
    }
    Ok(())
}

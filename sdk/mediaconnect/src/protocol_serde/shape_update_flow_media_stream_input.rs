// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_flow_media_stream_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_flow_media_stream::UpdateFlowMediaStreamInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.attributes {
        #[allow(unused_mut)]
        let mut object_2 = object.key("attributes").start_object();
        crate::protocol_serde::shape_media_stream_attributes_request::ser_media_stream_attributes_request(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.clock_rate {
        object.key("clockRate").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_3).into()),
        );
    }
    if let Some(var_4) = &input.description {
        object.key("description").string(var_4.as_str());
    }
    if let Some(var_5) = &input.media_stream_type {
        object.key("mediaStreamType").string(var_5.as_str());
    }
    if let Some(var_6) = &input.video_format {
        object.key("videoFormat").string(var_6.as_str());
    }
    Ok(())
}

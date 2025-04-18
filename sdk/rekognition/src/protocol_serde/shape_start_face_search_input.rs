// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_start_face_search_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::start_face_search::StartFaceSearchInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.video {
        #[allow(unused_mut)]
        let mut object_2 = object.key("Video").start_object();
        crate::protocol_serde::shape_video::ser_video(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.client_request_token {
        object.key("ClientRequestToken").string(var_3.as_str());
    }
    if let Some(var_4) = &input.face_match_threshold {
        object.key("FaceMatchThreshold").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::Float((*var_4).into()),
        );
    }
    if let Some(var_5) = &input.collection_id {
        object.key("CollectionId").string(var_5.as_str());
    }
    if let Some(var_6) = &input.notification_channel {
        #[allow(unused_mut)]
        let mut object_7 = object.key("NotificationChannel").start_object();
        crate::protocol_serde::shape_notification_channel::ser_notification_channel(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.job_tag {
        object.key("JobTag").string(var_8.as_str());
    }
    Ok(())
}

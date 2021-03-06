// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn serialize_structure_crate_input_create_media_capture_pipeline_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::CreateMediaCapturePipelineInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_1) = &input.chime_sdk_meeting_configuration {
        let mut object_2 = object.key("ChimeSdkMeetingConfiguration").start_object();
        crate::json_ser::serialize_structure_crate_model_chime_sdk_meeting_configuration(
            &mut object_2,
            var_1,
        )?;
        object_2.finish();
    }
    if let Some(var_3) = &input.client_request_token {
        object.key("ClientRequestToken").string(var_3.as_str());
    }
    if let Some(var_4) = &input.sink_arn {
        object.key("SinkArn").string(var_4.as_str());
    }
    if let Some(var_5) = &input.sink_type {
        object.key("SinkType").string(var_5.as_str());
    }
    if let Some(var_6) = &input.source_arn {
        object.key("SourceArn").string(var_6.as_str());
    }
    if let Some(var_7) = &input.source_type {
        object.key("SourceType").string(var_7.as_str());
    }
    if let Some(var_8) = &input.tags {
        let mut array_9 = object.key("Tags").start_array();
        for item_10 in var_8 {
            {
                let mut object_11 = array_9.value().start_object();
                crate::json_ser::serialize_structure_crate_model_tag(&mut object_11, item_10)?;
                object_11.finish();
            }
        }
        array_9.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_tag_resource_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::TagResourceInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_12) = &input.resource_arn {
        object.key("ResourceARN").string(var_12.as_str());
    }
    if let Some(var_13) = &input.tags {
        let mut array_14 = object.key("Tags").start_array();
        for item_15 in var_13 {
            {
                let mut object_16 = array_14.value().start_object();
                crate::json_ser::serialize_structure_crate_model_tag(&mut object_16, item_15)?;
                object_16.finish();
            }
        }
        array_14.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_untag_resource_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::UntagResourceInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_17) = &input.resource_arn {
        object.key("ResourceARN").string(var_17.as_str());
    }
    if let Some(var_18) = &input.tag_keys {
        let mut array_19 = object.key("TagKeys").start_array();
        for item_20 in var_18 {
            {
                array_19.value().string(item_20.as_str());
            }
        }
        array_19.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_model_chime_sdk_meeting_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::ChimeSdkMeetingConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_21) = &input.source_configuration {
        let mut object_22 = object.key("SourceConfiguration").start_object();
        crate::json_ser::serialize_structure_crate_model_source_configuration(
            &mut object_22,
            var_21,
        )?;
        object_22.finish();
    }
    if let Some(var_23) = &input.artifacts_configuration {
        let mut object_24 = object.key("ArtifactsConfiguration").start_object();
        crate::json_ser::serialize_structure_crate_model_artifacts_configuration(
            &mut object_24,
            var_23,
        )?;
        object_24.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_model_tag(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::Tag,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_25) = &input.key {
        object.key("Key").string(var_25.as_str());
    }
    if let Some(var_26) = &input.value {
        object.key("Value").string(var_26.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_source_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::SourceConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_27) = &input.selected_video_streams {
        let mut object_28 = object.key("SelectedVideoStreams").start_object();
        crate::json_ser::serialize_structure_crate_model_selected_video_streams(
            &mut object_28,
            var_27,
        )?;
        object_28.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_model_artifacts_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::ArtifactsConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_29) = &input.audio {
        let mut object_30 = object.key("Audio").start_object();
        crate::json_ser::serialize_structure_crate_model_audio_artifacts_configuration(
            &mut object_30,
            var_29,
        )?;
        object_30.finish();
    }
    if let Some(var_31) = &input.video {
        let mut object_32 = object.key("Video").start_object();
        crate::json_ser::serialize_structure_crate_model_video_artifacts_configuration(
            &mut object_32,
            var_31,
        )?;
        object_32.finish();
    }
    if let Some(var_33) = &input.content {
        let mut object_34 = object.key("Content").start_object();
        crate::json_ser::serialize_structure_crate_model_content_artifacts_configuration(
            &mut object_34,
            var_33,
        )?;
        object_34.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_model_selected_video_streams(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::SelectedVideoStreams,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_35) = &input.attendee_ids {
        let mut array_36 = object.key("AttendeeIds").start_array();
        for item_37 in var_35 {
            {
                array_36.value().string(item_37.as_str());
            }
        }
        array_36.finish();
    }
    if let Some(var_38) = &input.external_user_ids {
        let mut array_39 = object.key("ExternalUserIds").start_array();
        for item_40 in var_38 {
            {
                array_39.value().string(item_40.as_str());
            }
        }
        array_39.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_model_audio_artifacts_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::AudioArtifactsConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_41) = &input.mux_type {
        object.key("MuxType").string(var_41.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_video_artifacts_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::VideoArtifactsConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_42) = &input.state {
        object.key("State").string(var_42.as_str());
    }
    if let Some(var_43) = &input.mux_type {
        object.key("MuxType").string(var_43.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_content_artifacts_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::ContentArtifactsConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_44) = &input.state {
        object.key("State").string(var_44.as_str());
    }
    if let Some(var_45) = &input.mux_type {
        object.key("MuxType").string(var_45.as_str());
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn serialize_structure_crate_input_associate_channel_flow_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::AssociateChannelFlowInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_1) = &input.channel_flow_arn {
        object.key("ChannelFlowArn").string(var_1.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_input_batch_create_channel_membership_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::BatchCreateChannelMembershipInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_2) = &input.member_arns {
        let mut array_3 = object.key("MemberArns").start_array();
        for item_4 in var_2 {
            {
                array_3.value().string(item_4.as_str());
            }
        }
        array_3.finish();
    }
    if let Some(var_5) = &input.r#type {
        object.key("Type").string(var_5.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_input_channel_flow_callback_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::ChannelFlowCallbackInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_6) = &input.callback_id {
        object.key("CallbackId").string(var_6.as_str());
    }
    if let Some(var_7) = &input.channel_message {
        let mut object_8 = object.key("ChannelMessage").start_object();
        crate::json_ser::serialize_structure_crate_model_channel_message_callback(
            &mut object_8,
            var_7,
        )?;
        object_8.finish();
    }
    if input.delete_resource {
        object.key("DeleteResource").boolean(input.delete_resource);
    }
    Ok(())
}

pub fn serialize_structure_crate_input_create_channel_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::CreateChannelInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_9) = &input.app_instance_arn {
        object.key("AppInstanceArn").string(var_9.as_str());
    }
    if let Some(var_10) = &input.channel_id {
        object.key("ChannelId").string(var_10.as_str());
    }
    if let Some(var_11) = &input.client_request_token {
        object.key("ClientRequestToken").string(var_11.as_str());
    }
    if let Some(var_12) = &input.member_arns {
        let mut array_13 = object.key("MemberArns").start_array();
        for item_14 in var_12 {
            {
                array_13.value().string(item_14.as_str());
            }
        }
        array_13.finish();
    }
    if let Some(var_15) = &input.metadata {
        object.key("Metadata").string(var_15.as_str());
    }
    if let Some(var_16) = &input.mode {
        object.key("Mode").string(var_16.as_str());
    }
    if let Some(var_17) = &input.moderator_arns {
        let mut array_18 = object.key("ModeratorArns").start_array();
        for item_19 in var_17 {
            {
                array_18.value().string(item_19.as_str());
            }
        }
        array_18.finish();
    }
    if let Some(var_20) = &input.name {
        object.key("Name").string(var_20.as_str());
    }
    if let Some(var_21) = &input.privacy {
        object.key("Privacy").string(var_21.as_str());
    }
    if let Some(var_22) = &input.tags {
        let mut array_23 = object.key("Tags").start_array();
        for item_24 in var_22 {
            {
                let mut object_25 = array_23.value().start_object();
                crate::json_ser::serialize_structure_crate_model_tag(&mut object_25, item_24)?;
                object_25.finish();
            }
        }
        array_23.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_create_channel_ban_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::CreateChannelBanInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_26) = &input.member_arn {
        object.key("MemberArn").string(var_26.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_input_create_channel_flow_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::CreateChannelFlowInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_27) = &input.app_instance_arn {
        object.key("AppInstanceArn").string(var_27.as_str());
    }
    if let Some(var_28) = &input.client_request_token {
        object.key("ClientRequestToken").string(var_28.as_str());
    }
    if let Some(var_29) = &input.name {
        object.key("Name").string(var_29.as_str());
    }
    if let Some(var_30) = &input.processors {
        let mut array_31 = object.key("Processors").start_array();
        for item_32 in var_30 {
            {
                let mut object_33 = array_31.value().start_object();
                crate::json_ser::serialize_structure_crate_model_processor(
                    &mut object_33,
                    item_32,
                )?;
                object_33.finish();
            }
        }
        array_31.finish();
    }
    if let Some(var_34) = &input.tags {
        let mut array_35 = object.key("Tags").start_array();
        for item_36 in var_34 {
            {
                let mut object_37 = array_35.value().start_object();
                crate::json_ser::serialize_structure_crate_model_tag(&mut object_37, item_36)?;
                object_37.finish();
            }
        }
        array_35.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_create_channel_membership_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::CreateChannelMembershipInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_38) = &input.member_arn {
        object.key("MemberArn").string(var_38.as_str());
    }
    if let Some(var_39) = &input.r#type {
        object.key("Type").string(var_39.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_input_create_channel_moderator_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::CreateChannelModeratorInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_40) = &input.channel_moderator_arn {
        object.key("ChannelModeratorArn").string(var_40.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_input_put_channel_membership_preferences_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::PutChannelMembershipPreferencesInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_41) = &input.preferences {
        let mut object_42 = object.key("Preferences").start_object();
        crate::json_ser::serialize_structure_crate_model_channel_membership_preferences(
            &mut object_42,
            var_41,
        )?;
        object_42.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_search_channels_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::SearchChannelsInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_43) = &input.fields {
        let mut array_44 = object.key("Fields").start_array();
        for item_45 in var_43 {
            {
                let mut object_46 = array_44.value().start_object();
                crate::json_ser::serialize_structure_crate_model_search_field(
                    &mut object_46,
                    item_45,
                )?;
                object_46.finish();
            }
        }
        array_44.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_send_channel_message_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::SendChannelMessageInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_47) = &input.client_request_token {
        object.key("ClientRequestToken").string(var_47.as_str());
    }
    if let Some(var_48) = &input.content {
        object.key("Content").string(var_48.as_str());
    }
    if let Some(var_49) = &input.message_attributes {
        let mut object_50 = object.key("MessageAttributes").start_object();
        for (key_51, value_52) in var_49 {
            {
                let mut object_53 = object_50.key(key_51).start_object();
                crate::json_ser::serialize_structure_crate_model_message_attribute_value(
                    &mut object_53,
                    value_52,
                )?;
                object_53.finish();
            }
        }
        object_50.finish();
    }
    if let Some(var_54) = &input.metadata {
        object.key("Metadata").string(var_54.as_str());
    }
    if let Some(var_55) = &input.persistence {
        object.key("Persistence").string(var_55.as_str());
    }
    if let Some(var_56) = &input.push_notification {
        let mut object_57 = object.key("PushNotification").start_object();
        crate::json_ser::serialize_structure_crate_model_push_notification_configuration(
            &mut object_57,
            var_56,
        )?;
        object_57.finish();
    }
    if let Some(var_58) = &input.r#type {
        object.key("Type").string(var_58.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_input_tag_resource_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::TagResourceInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_59) = &input.resource_arn {
        object.key("ResourceARN").string(var_59.as_str());
    }
    if let Some(var_60) = &input.tags {
        let mut array_61 = object.key("Tags").start_array();
        for item_62 in var_60 {
            {
                let mut object_63 = array_61.value().start_object();
                crate::json_ser::serialize_structure_crate_model_tag(&mut object_63, item_62)?;
                object_63.finish();
            }
        }
        array_61.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_untag_resource_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::UntagResourceInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_64) = &input.resource_arn {
        object.key("ResourceARN").string(var_64.as_str());
    }
    if let Some(var_65) = &input.tag_keys {
        let mut array_66 = object.key("TagKeys").start_array();
        for item_67 in var_65 {
            {
                array_66.value().string(item_67.as_str());
            }
        }
        array_66.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_update_channel_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::UpdateChannelInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_68) = &input.metadata {
        object.key("Metadata").string(var_68.as_str());
    }
    if let Some(var_69) = &input.mode {
        object.key("Mode").string(var_69.as_str());
    }
    if let Some(var_70) = &input.name {
        object.key("Name").string(var_70.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_input_update_channel_flow_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::UpdateChannelFlowInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_71) = &input.name {
        object.key("Name").string(var_71.as_str());
    }
    if let Some(var_72) = &input.processors {
        let mut array_73 = object.key("Processors").start_array();
        for item_74 in var_72 {
            {
                let mut object_75 = array_73.value().start_object();
                crate::json_ser::serialize_structure_crate_model_processor(
                    &mut object_75,
                    item_74,
                )?;
                object_75.finish();
            }
        }
        array_73.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_update_channel_message_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::UpdateChannelMessageInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_76) = &input.content {
        object.key("Content").string(var_76.as_str());
    }
    if let Some(var_77) = &input.metadata {
        object.key("Metadata").string(var_77.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_channel_message_callback(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::ChannelMessageCallback,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_78) = &input.message_id {
        object.key("MessageId").string(var_78.as_str());
    }
    if let Some(var_79) = &input.content {
        object.key("Content").string(var_79.as_str());
    }
    if let Some(var_80) = &input.metadata {
        object.key("Metadata").string(var_80.as_str());
    }
    if let Some(var_81) = &input.push_notification {
        let mut object_82 = object.key("PushNotification").start_object();
        crate::json_ser::serialize_structure_crate_model_push_notification_configuration(
            &mut object_82,
            var_81,
        )?;
        object_82.finish();
    }
    if let Some(var_83) = &input.message_attributes {
        let mut object_84 = object.key("MessageAttributes").start_object();
        for (key_85, value_86) in var_83 {
            {
                let mut object_87 = object_84.key(key_85).start_object();
                crate::json_ser::serialize_structure_crate_model_message_attribute_value(
                    &mut object_87,
                    value_86,
                )?;
                object_87.finish();
            }
        }
        object_84.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_model_tag(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::Tag,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_88) = &input.key {
        object.key("Key").string(var_88.as_str());
    }
    if let Some(var_89) = &input.value {
        object.key("Value").string(var_89.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_processor(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::Processor,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_90) = &input.name {
        object.key("Name").string(var_90.as_str());
    }
    if let Some(var_91) = &input.configuration {
        let mut object_92 = object.key("Configuration").start_object();
        crate::json_ser::serialize_structure_crate_model_processor_configuration(
            &mut object_92,
            var_91,
        )?;
        object_92.finish();
    }
    if let Some(var_93) = &input.execution_order {
        object.key("ExecutionOrder").number(
            #[allow(clippy::useless_conversion)]
            aws_smithy_types::Number::NegInt((*var_93).into()),
        );
    }
    if let Some(var_94) = &input.fallback_action {
        object.key("FallbackAction").string(var_94.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_channel_membership_preferences(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::ChannelMembershipPreferences,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_95) = &input.push_notifications {
        let mut object_96 = object.key("PushNotifications").start_object();
        crate::json_ser::serialize_structure_crate_model_push_notification_preferences(
            &mut object_96,
            var_95,
        )?;
        object_96.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_model_search_field(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::SearchField,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_97) = &input.key {
        object.key("Key").string(var_97.as_str());
    }
    if let Some(var_98) = &input.values {
        let mut array_99 = object.key("Values").start_array();
        for item_100 in var_98 {
            {
                array_99.value().string(item_100.as_str());
            }
        }
        array_99.finish();
    }
    if let Some(var_101) = &input.operator {
        object.key("Operator").string(var_101.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_message_attribute_value(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::MessageAttributeValue,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_102) = &input.string_values {
        let mut array_103 = object.key("StringValues").start_array();
        for item_104 in var_102 {
            {
                array_103.value().string(item_104.as_str());
            }
        }
        array_103.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_model_push_notification_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::PushNotificationConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_105) = &input.title {
        object.key("Title").string(var_105.as_str());
    }
    if let Some(var_106) = &input.body {
        object.key("Body").string(var_106.as_str());
    }
    if let Some(var_107) = &input.r#type {
        object.key("Type").string(var_107.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_processor_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::ProcessorConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_108) = &input.lambda {
        let mut object_109 = object.key("Lambda").start_object();
        crate::json_ser::serialize_structure_crate_model_lambda_configuration(
            &mut object_109,
            var_108,
        )?;
        object_109.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_model_push_notification_preferences(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::PushNotificationPreferences,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_110) = &input.allow_notifications {
        object.key("AllowNotifications").string(var_110.as_str());
    }
    if let Some(var_111) = &input.filter_rule {
        object.key("FilterRule").string(var_111.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_lambda_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::LambdaConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_112) = &input.resource_arn {
        object.key("ResourceArn").string(var_112.as_str());
    }
    if let Some(var_113) = &input.invocation_type {
        object.key("InvocationType").string(var_113.as_str());
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn serialize_structure_crate_input_accept_eulas_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::AcceptEulasInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_1) = &input.eula_ids {
        let mut array_2 = object.key("eulaIds").start_array();
        for item_3 in var_1 {
            {
                array_2.value().string(item_3.as_str());
            }
        }
        array_2.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_create_launch_profile_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::CreateLaunchProfileInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_4) = &input.description {
        object.key("description").string(var_4.as_str());
    }
    if let Some(var_5) = &input.ec2_subnet_ids {
        let mut array_6 = object.key("ec2SubnetIds").start_array();
        for item_7 in var_5 {
            {
                array_6.value().string(item_7.as_str());
            }
        }
        array_6.finish();
    }
    if let Some(var_8) = &input.launch_profile_protocol_versions {
        let mut array_9 = object.key("launchProfileProtocolVersions").start_array();
        for item_10 in var_8 {
            {
                array_9.value().string(item_10.as_str());
            }
        }
        array_9.finish();
    }
    if let Some(var_11) = &input.name {
        object.key("name").string(var_11.as_str());
    }
    if let Some(var_12) = &input.stream_configuration {
        let mut object_13 = object.key("streamConfiguration").start_object();
        crate::json_ser::serialize_structure_crate_model_stream_configuration_create(
            &mut object_13,
            var_12,
        )?;
        object_13.finish();
    }
    if let Some(var_14) = &input.studio_component_ids {
        let mut array_15 = object.key("studioComponentIds").start_array();
        for item_16 in var_14 {
            {
                array_15.value().string(item_16.as_str());
            }
        }
        array_15.finish();
    }
    if let Some(var_17) = &input.tags {
        let mut object_18 = object.key("tags").start_object();
        for (key_19, value_20) in var_17 {
            {
                object_18.key(key_19).string(value_20.as_str());
            }
        }
        object_18.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_create_streaming_image_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::CreateStreamingImageInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_21) = &input.description {
        object.key("description").string(var_21.as_str());
    }
    if let Some(var_22) = &input.ec2_image_id {
        object.key("ec2ImageId").string(var_22.as_str());
    }
    if let Some(var_23) = &input.name {
        object.key("name").string(var_23.as_str());
    }
    if let Some(var_24) = &input.tags {
        let mut object_25 = object.key("tags").start_object();
        for (key_26, value_27) in var_24 {
            {
                object_25.key(key_26).string(value_27.as_str());
            }
        }
        object_25.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_create_streaming_session_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::CreateStreamingSessionInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_28) = &input.ec2_instance_type {
        object.key("ec2InstanceType").string(var_28.as_str());
    }
    if let Some(var_29) = &input.launch_profile_id {
        object.key("launchProfileId").string(var_29.as_str());
    }
    if let Some(var_30) = &input.owned_by {
        object.key("ownedBy").string(var_30.as_str());
    }
    if let Some(var_31) = &input.streaming_image_id {
        object.key("streamingImageId").string(var_31.as_str());
    }
    if let Some(var_32) = &input.tags {
        let mut object_33 = object.key("tags").start_object();
        for (key_34, value_35) in var_32 {
            {
                object_33.key(key_34).string(value_35.as_str());
            }
        }
        object_33.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_create_streaming_session_stream_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::CreateStreamingSessionStreamInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if input.expiration_in_seconds != 0 {
        object.key("expirationInSeconds").number(
            #[allow(clippy::useless_conversion)]
            aws_smithy_types::Number::NegInt((input.expiration_in_seconds).into()),
        );
    }
    Ok(())
}

pub fn serialize_structure_crate_input_create_studio_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::CreateStudioInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_36) = &input.admin_role_arn {
        object.key("adminRoleArn").string(var_36.as_str());
    }
    if let Some(var_37) = &input.display_name {
        object.key("displayName").string(var_37.as_str());
    }
    if let Some(var_38) = &input.studio_encryption_configuration {
        let mut object_39 = object.key("studioEncryptionConfiguration").start_object();
        crate::json_ser::serialize_structure_crate_model_studio_encryption_configuration(
            &mut object_39,
            var_38,
        )?;
        object_39.finish();
    }
    if let Some(var_40) = &input.studio_name {
        object.key("studioName").string(var_40.as_str());
    }
    if let Some(var_41) = &input.tags {
        let mut object_42 = object.key("tags").start_object();
        for (key_43, value_44) in var_41 {
            {
                object_42.key(key_43).string(value_44.as_str());
            }
        }
        object_42.finish();
    }
    if let Some(var_45) = &input.user_role_arn {
        object.key("userRoleArn").string(var_45.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_input_create_studio_component_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::CreateStudioComponentInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_46) = &input.configuration {
        let mut object_47 = object.key("configuration").start_object();
        crate::json_ser::serialize_union_crate_model_studio_component_configuration(
            &mut object_47,
            var_46,
        )?;
        object_47.finish();
    }
    if let Some(var_48) = &input.description {
        object.key("description").string(var_48.as_str());
    }
    if let Some(var_49) = &input.ec2_security_group_ids {
        let mut array_50 = object.key("ec2SecurityGroupIds").start_array();
        for item_51 in var_49 {
            {
                array_50.value().string(item_51.as_str());
            }
        }
        array_50.finish();
    }
    if let Some(var_52) = &input.initialization_scripts {
        let mut array_53 = object.key("initializationScripts").start_array();
        for item_54 in var_52 {
            {
                let mut object_55 = array_53.value().start_object();
                crate::json_ser::serialize_structure_crate_model_studio_component_initialization_script(&mut object_55, item_54)?;
                object_55.finish();
            }
        }
        array_53.finish();
    }
    if let Some(var_56) = &input.name {
        object.key("name").string(var_56.as_str());
    }
    if let Some(var_57) = &input.runtime_role_arn {
        object.key("runtimeRoleArn").string(var_57.as_str());
    }
    if let Some(var_58) = &input.script_parameters {
        let mut array_59 = object.key("scriptParameters").start_array();
        for item_60 in var_58 {
            {
                let mut object_61 = array_59.value().start_object();
                crate::json_ser::serialize_structure_crate_model_script_parameter_key_value(
                    &mut object_61,
                    item_60,
                )?;
                object_61.finish();
            }
        }
        array_59.finish();
    }
    if let Some(var_62) = &input.secure_initialization_role_arn {
        object
            .key("secureInitializationRoleArn")
            .string(var_62.as_str());
    }
    if let Some(var_63) = &input.subtype {
        object.key("subtype").string(var_63.as_str());
    }
    if let Some(var_64) = &input.tags {
        let mut object_65 = object.key("tags").start_object();
        for (key_66, value_67) in var_64 {
            {
                object_65.key(key_66).string(value_67.as_str());
            }
        }
        object_65.finish();
    }
    if let Some(var_68) = &input.r#type {
        object.key("type").string(var_68.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_input_put_launch_profile_members_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::PutLaunchProfileMembersInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_69) = &input.identity_store_id {
        object.key("identityStoreId").string(var_69.as_str());
    }
    if let Some(var_70) = &input.members {
        let mut array_71 = object.key("members").start_array();
        for item_72 in var_70 {
            {
                let mut object_73 = array_71.value().start_object();
                crate::json_ser::serialize_structure_crate_model_new_launch_profile_member(
                    &mut object_73,
                    item_72,
                )?;
                object_73.finish();
            }
        }
        array_71.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_put_studio_members_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::PutStudioMembersInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_74) = &input.identity_store_id {
        object.key("identityStoreId").string(var_74.as_str());
    }
    if let Some(var_75) = &input.members {
        let mut array_76 = object.key("members").start_array();
        for item_77 in var_75 {
            {
                let mut object_78 = array_76.value().start_object();
                crate::json_ser::serialize_structure_crate_model_new_studio_member(
                    &mut object_78,
                    item_77,
                )?;
                object_78.finish();
            }
        }
        array_76.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_tag_resource_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::TagResourceInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_79) = &input.tags {
        let mut object_80 = object.key("tags").start_object();
        for (key_81, value_82) in var_79 {
            {
                object_80.key(key_81).string(value_82.as_str());
            }
        }
        object_80.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_update_launch_profile_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::UpdateLaunchProfileInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_83) = &input.description {
        object.key("description").string(var_83.as_str());
    }
    if let Some(var_84) = &input.launch_profile_protocol_versions {
        let mut array_85 = object.key("launchProfileProtocolVersions").start_array();
        for item_86 in var_84 {
            {
                array_85.value().string(item_86.as_str());
            }
        }
        array_85.finish();
    }
    if let Some(var_87) = &input.name {
        object.key("name").string(var_87.as_str());
    }
    if let Some(var_88) = &input.stream_configuration {
        let mut object_89 = object.key("streamConfiguration").start_object();
        crate::json_ser::serialize_structure_crate_model_stream_configuration_create(
            &mut object_89,
            var_88,
        )?;
        object_89.finish();
    }
    if let Some(var_90) = &input.studio_component_ids {
        let mut array_91 = object.key("studioComponentIds").start_array();
        for item_92 in var_90 {
            {
                array_91.value().string(item_92.as_str());
            }
        }
        array_91.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_input_update_launch_profile_member_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::UpdateLaunchProfileMemberInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_93) = &input.persona {
        object.key("persona").string(var_93.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_input_update_streaming_image_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::UpdateStreamingImageInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_94) = &input.description {
        object.key("description").string(var_94.as_str());
    }
    if let Some(var_95) = &input.name {
        object.key("name").string(var_95.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_input_update_studio_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::UpdateStudioInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_96) = &input.admin_role_arn {
        object.key("adminRoleArn").string(var_96.as_str());
    }
    if let Some(var_97) = &input.display_name {
        object.key("displayName").string(var_97.as_str());
    }
    if let Some(var_98) = &input.user_role_arn {
        object.key("userRoleArn").string(var_98.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_input_update_studio_component_input(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::input::UpdateStudioComponentInput,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_99) = &input.configuration {
        let mut object_100 = object.key("configuration").start_object();
        crate::json_ser::serialize_union_crate_model_studio_component_configuration(
            &mut object_100,
            var_99,
        )?;
        object_100.finish();
    }
    if let Some(var_101) = &input.description {
        object.key("description").string(var_101.as_str());
    }
    if let Some(var_102) = &input.ec2_security_group_ids {
        let mut array_103 = object.key("ec2SecurityGroupIds").start_array();
        for item_104 in var_102 {
            {
                array_103.value().string(item_104.as_str());
            }
        }
        array_103.finish();
    }
    if let Some(var_105) = &input.initialization_scripts {
        let mut array_106 = object.key("initializationScripts").start_array();
        for item_107 in var_105 {
            {
                let mut object_108 = array_106.value().start_object();
                crate::json_ser::serialize_structure_crate_model_studio_component_initialization_script(&mut object_108, item_107)?;
                object_108.finish();
            }
        }
        array_106.finish();
    }
    if let Some(var_109) = &input.name {
        object.key("name").string(var_109.as_str());
    }
    if let Some(var_110) = &input.runtime_role_arn {
        object.key("runtimeRoleArn").string(var_110.as_str());
    }
    if let Some(var_111) = &input.script_parameters {
        let mut array_112 = object.key("scriptParameters").start_array();
        for item_113 in var_111 {
            {
                let mut object_114 = array_112.value().start_object();
                crate::json_ser::serialize_structure_crate_model_script_parameter_key_value(
                    &mut object_114,
                    item_113,
                )?;
                object_114.finish();
            }
        }
        array_112.finish();
    }
    if let Some(var_115) = &input.secure_initialization_role_arn {
        object
            .key("secureInitializationRoleArn")
            .string(var_115.as_str());
    }
    if let Some(var_116) = &input.subtype {
        object.key("subtype").string(var_116.as_str());
    }
    if let Some(var_117) = &input.r#type {
        object.key("type").string(var_117.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_stream_configuration_create(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::StreamConfigurationCreate,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_118) = &input.clipboard_mode {
        object.key("clipboardMode").string(var_118.as_str());
    }
    if let Some(var_119) = &input.ec2_instance_types {
        let mut array_120 = object.key("ec2InstanceTypes").start_array();
        for item_121 in var_119 {
            {
                array_120.value().string(item_121.as_str());
            }
        }
        array_120.finish();
    }
    if input.max_session_length_in_minutes != 0 {
        object.key("maxSessionLengthInMinutes").number(
            #[allow(clippy::useless_conversion)]
            aws_smithy_types::Number::NegInt((input.max_session_length_in_minutes).into()),
        );
    }
    if let Some(var_122) = &input.streaming_image_ids {
        let mut array_123 = object.key("streamingImageIds").start_array();
        for item_124 in var_122 {
            {
                array_123.value().string(item_124.as_str());
            }
        }
        array_123.finish();
    }
    if input.max_stopped_session_length_in_minutes != 0 {
        object.key("maxStoppedSessionLengthInMinutes").number(
            #[allow(clippy::useless_conversion)]
            aws_smithy_types::Number::NegInt((input.max_stopped_session_length_in_minutes).into()),
        );
    }
    if let Some(var_125) = &input.session_storage {
        let mut object_126 = object.key("sessionStorage").start_object();
        crate::json_ser::serialize_structure_crate_model_stream_configuration_session_storage(
            &mut object_126,
            var_125,
        )?;
        object_126.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_model_studio_encryption_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::StudioEncryptionConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_127) = &input.key_arn {
        object.key("keyArn").string(var_127.as_str());
    }
    if let Some(var_128) = &input.key_type {
        object.key("keyType").string(var_128.as_str());
    }
    Ok(())
}

pub fn serialize_union_crate_model_studio_component_configuration(
    object_47: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::StudioComponentConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    match input {
        crate::model::StudioComponentConfiguration::ActiveDirectoryConfiguration(inner) => {
            let mut object_129 = object_47.key("activeDirectoryConfiguration").start_object();
            crate::json_ser::serialize_structure_crate_model_active_directory_configuration(
                &mut object_129,
                inner,
            )?;
            object_129.finish();
        }
        crate::model::StudioComponentConfiguration::ComputeFarmConfiguration(inner) => {
            let mut object_130 = object_47.key("computeFarmConfiguration").start_object();
            crate::json_ser::serialize_structure_crate_model_compute_farm_configuration(
                &mut object_130,
                inner,
            )?;
            object_130.finish();
        }
        crate::model::StudioComponentConfiguration::LicenseServiceConfiguration(inner) => {
            let mut object_131 = object_47.key("licenseServiceConfiguration").start_object();
            crate::json_ser::serialize_structure_crate_model_license_service_configuration(
                &mut object_131,
                inner,
            )?;
            object_131.finish();
        }
        crate::model::StudioComponentConfiguration::SharedFileSystemConfiguration(inner) => {
            let mut object_132 = object_47
                .key("sharedFileSystemConfiguration")
                .start_object();
            crate::json_ser::serialize_structure_crate_model_shared_file_system_configuration(
                &mut object_132,
                inner,
            )?;
            object_132.finish();
        }
        crate::model::StudioComponentConfiguration::Unknown => {
            return Err(
                aws_smithy_http::operation::SerializationError::unknown_variant(
                    "StudioComponentConfiguration",
                ),
            )
        }
    }
    Ok(())
}

pub fn serialize_structure_crate_model_studio_component_initialization_script(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::StudioComponentInitializationScript,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_133) = &input.launch_profile_protocol_version {
        object
            .key("launchProfileProtocolVersion")
            .string(var_133.as_str());
    }
    if let Some(var_134) = &input.platform {
        object.key("platform").string(var_134.as_str());
    }
    if let Some(var_135) = &input.run_context {
        object.key("runContext").string(var_135.as_str());
    }
    if let Some(var_136) = &input.script {
        object.key("script").string(var_136.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_script_parameter_key_value(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::ScriptParameterKeyValue,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_137) = &input.key {
        object.key("key").string(var_137.as_str());
    }
    if let Some(var_138) = &input.value {
        object.key("value").string(var_138.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_new_launch_profile_member(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::NewLaunchProfileMember,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_139) = &input.persona {
        object.key("persona").string(var_139.as_str());
    }
    if let Some(var_140) = &input.principal_id {
        object.key("principalId").string(var_140.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_new_studio_member(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::NewStudioMember,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_141) = &input.persona {
        object.key("persona").string(var_141.as_str());
    }
    if let Some(var_142) = &input.principal_id {
        object.key("principalId").string(var_142.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_stream_configuration_session_storage(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::StreamConfigurationSessionStorage,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_143) = &input.root {
        let mut object_144 = object.key("root").start_object();
        crate::json_ser::serialize_structure_crate_model_streaming_session_storage_root(
            &mut object_144,
            var_143,
        )?;
        object_144.finish();
    }
    if let Some(var_145) = &input.mode {
        let mut array_146 = object.key("mode").start_array();
        for item_147 in var_145 {
            {
                array_146.value().string(item_147.as_str());
            }
        }
        array_146.finish();
    }
    Ok(())
}

pub fn serialize_structure_crate_model_active_directory_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::ActiveDirectoryConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_148) = &input.computer_attributes {
        let mut array_149 = object.key("computerAttributes").start_array();
        for item_150 in var_148 {
            {
                let mut object_151 = array_149.value().start_object();
                crate::json_ser::serialize_structure_crate_model_active_directory_computer_attribute(&mut object_151, item_150)?;
                object_151.finish();
            }
        }
        array_149.finish();
    }
    if let Some(var_152) = &input.directory_id {
        object.key("directoryId").string(var_152.as_str());
    }
    if let Some(var_153) = &input.organizational_unit_distinguished_name {
        object
            .key("organizationalUnitDistinguishedName")
            .string(var_153.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_compute_farm_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::ComputeFarmConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_154) = &input.active_directory_user {
        object.key("activeDirectoryUser").string(var_154.as_str());
    }
    if let Some(var_155) = &input.endpoint {
        object.key("endpoint").string(var_155.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_license_service_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::LicenseServiceConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_156) = &input.endpoint {
        object.key("endpoint").string(var_156.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_shared_file_system_configuration(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::SharedFileSystemConfiguration,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_157) = &input.endpoint {
        object.key("endpoint").string(var_157.as_str());
    }
    if let Some(var_158) = &input.file_system_id {
        object.key("fileSystemId").string(var_158.as_str());
    }
    if let Some(var_159) = &input.linux_mount_point {
        object.key("linuxMountPoint").string(var_159.as_str());
    }
    if let Some(var_160) = &input.share_name {
        object.key("shareName").string(var_160.as_str());
    }
    if let Some(var_161) = &input.windows_mount_drive {
        object.key("windowsMountDrive").string(var_161.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_streaming_session_storage_root(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::StreamingSessionStorageRoot,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_162) = &input.linux {
        object.key("linux").string(var_162.as_str());
    }
    if let Some(var_163) = &input.windows {
        object.key("windows").string(var_163.as_str());
    }
    Ok(())
}

pub fn serialize_structure_crate_model_active_directory_computer_attribute(
    object: &mut aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::model::ActiveDirectoryComputerAttribute,
) -> Result<(), aws_smithy_http::operation::SerializationError> {
    if let Some(var_164) = &input.name {
        object.key("name").string(var_164.as_str());
    }
    if let Some(var_165) = &input.value {
        object.key("value").string(var_165.as_str());
    }
    Ok(())
}

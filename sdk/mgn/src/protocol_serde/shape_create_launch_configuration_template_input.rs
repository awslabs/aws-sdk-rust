// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_launch_configuration_template_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_launch_configuration_template::CreateLaunchConfigurationTemplateInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.associate_public_ip_address {
        object.key("associatePublicIpAddress").boolean(*var_1);
    }
    if let Some(var_2) = &input.boot_mode {
        object.key("bootMode").string(var_2.as_str());
    }
    if let Some(var_3) = &input.copy_private_ip {
        object.key("copyPrivateIp").boolean(*var_3);
    }
    if let Some(var_4) = &input.copy_tags {
        object.key("copyTags").boolean(*var_4);
    }
    if let Some(var_5) = &input.enable_map_auto_tagging {
        object.key("enableMapAutoTagging").boolean(*var_5);
    }
    if let Some(var_6) = &input.large_volume_conf {
        #[allow(unused_mut)]
        let mut object_7 = object.key("largeVolumeConf").start_object();
        crate::protocol_serde::shape_launch_template_disk_conf::ser_launch_template_disk_conf(&mut object_7, var_6)?;
        object_7.finish();
    }
    if let Some(var_8) = &input.launch_disposition {
        object.key("launchDisposition").string(var_8.as_str());
    }
    if let Some(var_9) = &input.licensing {
        #[allow(unused_mut)]
        let mut object_10 = object.key("licensing").start_object();
        crate::protocol_serde::shape_licensing::ser_licensing(&mut object_10, var_9)?;
        object_10.finish();
    }
    if let Some(var_11) = &input.map_auto_tagging_mpe_id {
        object.key("mapAutoTaggingMpeID").string(var_11.as_str());
    }
    if let Some(var_12) = &input.post_launch_actions {
        #[allow(unused_mut)]
        let mut object_13 = object.key("postLaunchActions").start_object();
        crate::protocol_serde::shape_post_launch_actions::ser_post_launch_actions(&mut object_13, var_12)?;
        object_13.finish();
    }
    if let Some(var_14) = &input.small_volume_conf {
        #[allow(unused_mut)]
        let mut object_15 = object.key("smallVolumeConf").start_object();
        crate::protocol_serde::shape_launch_template_disk_conf::ser_launch_template_disk_conf(&mut object_15, var_14)?;
        object_15.finish();
    }
    if let Some(var_16) = &input.small_volume_max_size {
        object.key("smallVolumeMaxSize").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_16).into()),
        );
    }
    if let Some(var_17) = &input.tags {
        #[allow(unused_mut)]
        let mut object_18 = object.key("tags").start_object();
        for (key_19, value_20) in var_17 {
            {
                object_18.key(key_19.as_str()).string(value_20.as_str());
            }
        }
        object_18.finish();
    }
    if let Some(var_21) = &input.target_instance_type_right_sizing_method {
        object.key("targetInstanceTypeRightSizingMethod").string(var_21.as_str());
    }
    Ok(())
}

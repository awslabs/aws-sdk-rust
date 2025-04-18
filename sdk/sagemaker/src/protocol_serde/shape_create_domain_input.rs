// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_domain_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_domain::CreateDomainInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.domain_name {
        object.key("DomainName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.auth_mode {
        object.key("AuthMode").string(var_2.as_str());
    }
    if let Some(var_3) = &input.default_user_settings {
        #[allow(unused_mut)]
        let mut object_4 = object.key("DefaultUserSettings").start_object();
        crate::protocol_serde::shape_user_settings::ser_user_settings(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.domain_settings {
        #[allow(unused_mut)]
        let mut object_6 = object.key("DomainSettings").start_object();
        crate::protocol_serde::shape_domain_settings::ser_domain_settings(&mut object_6, var_5)?;
        object_6.finish();
    }
    if let Some(var_7) = &input.subnet_ids {
        let mut array_8 = object.key("SubnetIds").start_array();
        for item_9 in var_7 {
            {
                array_8.value().string(item_9.as_str());
            }
        }
        array_8.finish();
    }
    if let Some(var_10) = &input.vpc_id {
        object.key("VpcId").string(var_10.as_str());
    }
    if let Some(var_11) = &input.tags {
        let mut array_12 = object.key("Tags").start_array();
        for item_13 in var_11 {
            {
                #[allow(unused_mut)]
                let mut object_14 = array_12.value().start_object();
                crate::protocol_serde::shape_tag::ser_tag(&mut object_14, item_13)?;
                object_14.finish();
            }
        }
        array_12.finish();
    }
    if let Some(var_15) = &input.app_network_access_type {
        object.key("AppNetworkAccessType").string(var_15.as_str());
    }
    if let Some(var_16) = &input.home_efs_file_system_kms_key_id {
        object.key("HomeEfsFileSystemKmsKeyId").string(var_16.as_str());
    }
    if let Some(var_17) = &input.kms_key_id {
        object.key("KmsKeyId").string(var_17.as_str());
    }
    if let Some(var_18) = &input.app_security_group_management {
        object.key("AppSecurityGroupManagement").string(var_18.as_str());
    }
    if let Some(var_19) = &input.tag_propagation {
        object.key("TagPropagation").string(var_19.as_str());
    }
    if let Some(var_20) = &input.default_space_settings {
        #[allow(unused_mut)]
        let mut object_21 = object.key("DefaultSpaceSettings").start_object();
        crate::protocol_serde::shape_default_space_settings::ser_default_space_settings(&mut object_21, var_20)?;
        object_21.finish();
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_namespace_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_namespace::UpdateNamespaceInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.namespace_name {
        object.key("namespaceName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.admin_user_password {
        object.key("adminUserPassword").string(var_2.as_str());
    }
    if let Some(var_3) = &input.admin_username {
        object.key("adminUsername").string(var_3.as_str());
    }
    if let Some(var_4) = &input.kms_key_id {
        object.key("kmsKeyId").string(var_4.as_str());
    }
    if let Some(var_5) = &input.default_iam_role_arn {
        object.key("defaultIamRoleArn").string(var_5.as_str());
    }
    if let Some(var_6) = &input.iam_roles {
        let mut array_7 = object.key("iamRoles").start_array();
        for item_8 in var_6 {
            {
                array_7.value().string(item_8.as_str());
            }
        }
        array_7.finish();
    }
    if let Some(var_9) = &input.log_exports {
        let mut array_10 = object.key("logExports").start_array();
        for item_11 in var_9 {
            {
                array_10.value().string(item_11.as_str());
            }
        }
        array_10.finish();
    }
    if let Some(var_12) = &input.manage_admin_password {
        object.key("manageAdminPassword").boolean(*var_12);
    }
    if let Some(var_13) = &input.admin_password_secret_kms_key_id {
        object.key("adminPasswordSecretKmsKeyId").string(var_13.as_str());
    }
    Ok(())
}

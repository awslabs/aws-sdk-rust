// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_grant_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_grant::CreateGrantInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.client_token {
        object.key("ClientToken").string(var_1.as_str());
    }
    if let Some(var_2) = &input.grant_name {
        object.key("GrantName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.license_arn {
        object.key("LicenseArn").string(var_3.as_str());
    }
    if let Some(var_4) = &input.principals {
        let mut array_5 = object.key("Principals").start_array();
        for item_6 in var_4 {
            {
                array_5.value().string(item_6.as_str());
            }
        }
        array_5.finish();
    }
    if let Some(var_7) = &input.home_region {
        object.key("HomeRegion").string(var_7.as_str());
    }
    if let Some(var_8) = &input.allowed_operations {
        let mut array_9 = object.key("AllowedOperations").start_array();
        for item_10 in var_8 {
            {
                array_9.value().string(item_10.as_str());
            }
        }
        array_9.finish();
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
    Ok(())
}

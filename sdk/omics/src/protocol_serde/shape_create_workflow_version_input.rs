// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_workflow_version_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_workflow_version::CreateWorkflowVersionInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.accelerators {
        object.key("accelerators").string(var_1.as_str());
    }
    if let Some(var_2) = &input.definition_repository {
        #[allow(unused_mut)]
        let mut object_3 = object.key("definitionRepository").start_object();
        crate::protocol_serde::shape_definition_repository::ser_definition_repository(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.definition_uri {
        object.key("definitionUri").string(var_4.as_str());
    }
    if let Some(var_5) = &input.definition_zip {
        object.key("definitionZip").string_unchecked(&::aws_smithy_types::base64::encode(var_5));
    }
    if let Some(var_6) = &input.description {
        object.key("description").string(var_6.as_str());
    }
    if let Some(var_7) = &input.engine {
        object.key("engine").string(var_7.as_str());
    }
    if let Some(var_8) = &input.main {
        object.key("main").string(var_8.as_str());
    }
    if let Some(var_9) = &input.parameter_template {
        #[allow(unused_mut)]
        let mut object_10 = object.key("parameterTemplate").start_object();
        for (key_11, value_12) in var_9 {
            {
                #[allow(unused_mut)]
                let mut object_13 = object_10.key(key_11.as_str()).start_object();
                crate::protocol_serde::shape_workflow_parameter::ser_workflow_parameter(&mut object_13, value_12)?;
                object_13.finish();
            }
        }
        object_10.finish();
    }
    if let Some(var_14) = &input.parameter_template_path {
        object.key("parameterTemplatePath").string(var_14.as_str());
    }
    if let Some(var_15) = &input.readme_markdown {
        object.key("readmeMarkdown").string(var_15.as_str());
    }
    if let Some(var_16) = &input.readme_path {
        object.key("readmePath").string(var_16.as_str());
    }
    if let Some(var_17) = &input.readme_uri {
        object.key("readmeUri").string(var_17.as_str());
    }
    if let Some(var_18) = &input.request_id {
        object.key("requestId").string(var_18.as_str());
    }
    if let Some(var_19) = &input.storage_capacity {
        object.key("storageCapacity").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_19).into()),
        );
    }
    if let Some(var_20) = &input.storage_type {
        object.key("storageType").string(var_20.as_str());
    }
    if let Some(var_21) = &input.tags {
        #[allow(unused_mut)]
        let mut object_22 = object.key("tags").start_object();
        for (key_23, value_24) in var_21 {
            {
                object_22.key(key_23.as_str()).string(value_24.as_str());
            }
        }
        object_22.finish();
    }
    if let Some(var_25) = &input.version_name {
        object.key("versionName").string(var_25.as_str());
    }
    if let Some(var_26) = &input.workflow_bucket_owner_id {
        object.key("workflowBucketOwnerId").string(var_26.as_str());
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_patch_baseline_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_patch_baseline::UpdatePatchBaselineInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.baseline_id {
        object.key("BaselineId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.name {
        object.key("Name").string(var_2.as_str());
    }
    if let Some(var_3) = &input.global_filters {
        #[allow(unused_mut)]
        let mut object_4 = object.key("GlobalFilters").start_object();
        crate::protocol_serde::shape_patch_filter_group::ser_patch_filter_group(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.approval_rules {
        #[allow(unused_mut)]
        let mut object_6 = object.key("ApprovalRules").start_object();
        crate::protocol_serde::shape_patch_rule_group::ser_patch_rule_group(&mut object_6, var_5)?;
        object_6.finish();
    }
    if let Some(var_7) = &input.approved_patches {
        let mut array_8 = object.key("ApprovedPatches").start_array();
        for item_9 in var_7 {
            {
                array_8.value().string(item_9.as_str());
            }
        }
        array_8.finish();
    }
    if let Some(var_10) = &input.approved_patches_compliance_level {
        object.key("ApprovedPatchesComplianceLevel").string(var_10.as_str());
    }
    if let Some(var_11) = &input.approved_patches_enable_non_security {
        object.key("ApprovedPatchesEnableNonSecurity").boolean(*var_11);
    }
    if let Some(var_12) = &input.rejected_patches {
        let mut array_13 = object.key("RejectedPatches").start_array();
        for item_14 in var_12 {
            {
                array_13.value().string(item_14.as_str());
            }
        }
        array_13.finish();
    }
    if let Some(var_15) = &input.rejected_patches_action {
        object.key("RejectedPatchesAction").string(var_15.as_str());
    }
    if let Some(var_16) = &input.description {
        object.key("Description").string(var_16.as_str());
    }
    if let Some(var_17) = &input.sources {
        let mut array_18 = object.key("Sources").start_array();
        for item_19 in var_17 {
            {
                #[allow(unused_mut)]
                let mut object_20 = array_18.value().start_object();
                crate::protocol_serde::shape_patch_source::ser_patch_source(&mut object_20, item_19)?;
                object_20.finish();
            }
        }
        array_18.finish();
    }
    if let Some(var_21) = &input.available_security_updates_compliance_status {
        object.key("AvailableSecurityUpdatesComplianceStatus").string(var_21.as_str());
    }
    if let Some(var_22) = &input.replace {
        object.key("Replace").boolean(*var_22);
    }
    Ok(())
}

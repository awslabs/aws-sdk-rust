// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn type_erase_result<O, E>(
    result: ::std::result::Result<O, E>,
) -> ::std::result::Result<
    ::aws_smithy_runtime_api::client::interceptors::context::Output,
    ::aws_smithy_runtime_api::client::orchestrator::OrchestratorError<::aws_smithy_runtime_api::client::interceptors::context::Error>,
>
where
    O: ::std::fmt::Debug + ::std::marker::Send + ::std::marker::Sync + 'static,
    E: ::std::error::Error + std::fmt::Debug + ::std::marker::Send + ::std::marker::Sync + 'static,
{
    result
        .map(|output| ::aws_smithy_runtime_api::client::interceptors::context::Output::erase(output))
        .map_err(|error| ::aws_smithy_runtime_api::client::interceptors::context::Error::erase(error))
        .map_err(::std::convert::Into::into)
}

pub fn parse_http_error_metadata(
    _response_status: u16,
    response_headers: &::aws_smithy_runtime_api::http::Headers,
    response_body: &[u8],
) -> ::std::result::Result<::aws_smithy_types::error::metadata::Builder, ::aws_smithy_json::deserialize::error::DeserializeError> {
    crate::json_errors::parse_error_metadata(response_body, response_headers)
}

pub(crate) mod shape_accept_page;

pub(crate) mod shape_activate_contact_channel;

pub(crate) mod shape_create_contact;

pub(crate) mod shape_create_contact_channel;

pub(crate) mod shape_create_rotation;

pub(crate) mod shape_create_rotation_override;

pub(crate) mod shape_deactivate_contact_channel;

pub(crate) mod shape_delete_contact;

pub(crate) mod shape_delete_contact_channel;

pub(crate) mod shape_delete_rotation;

pub(crate) mod shape_delete_rotation_override;

pub(crate) mod shape_describe_engagement;

pub(crate) mod shape_describe_page;

pub(crate) mod shape_get_contact;

pub(crate) mod shape_get_contact_channel;

pub(crate) mod shape_get_contact_policy;

pub(crate) mod shape_get_rotation;

pub(crate) mod shape_get_rotation_override;

pub(crate) mod shape_list_contact_channels;

pub(crate) mod shape_list_contacts;

pub(crate) mod shape_list_engagements;

pub(crate) mod shape_list_page_receipts;

pub(crate) mod shape_list_page_resolutions;

pub(crate) mod shape_list_pages_by_contact;

pub(crate) mod shape_list_pages_by_engagement;

pub(crate) mod shape_list_preview_rotation_shifts;

pub(crate) mod shape_list_rotation_overrides;

pub(crate) mod shape_list_rotation_shifts;

pub(crate) mod shape_list_rotations;

pub(crate) mod shape_list_tags_for_resource;

pub(crate) mod shape_put_contact_policy;

pub(crate) mod shape_send_activation_code;

pub(crate) mod shape_start_engagement;

pub(crate) mod shape_stop_engagement;

pub(crate) mod shape_tag_resource;

pub(crate) mod shape_untag_resource;

pub(crate) mod shape_update_contact;

pub(crate) mod shape_update_contact_channel;

pub(crate) mod shape_update_rotation;

pub(crate) mod shape_accept_page_input;

pub(crate) fn or_empty_doc(data: &[u8]) -> &[u8] {
    if data.is_empty() {
        b"{}"
    } else {
        data
    }
}

pub(crate) mod shape_access_denied_exception;

pub(crate) mod shape_activate_contact_channel_input;

pub(crate) mod shape_conflict_exception;

pub(crate) mod shape_create_contact_channel_input;

pub(crate) mod shape_create_contact_input;

pub(crate) mod shape_create_rotation_input;

pub(crate) mod shape_create_rotation_override_input;

pub(crate) mod shape_data_encryption_exception;

pub(crate) mod shape_deactivate_contact_channel_input;

pub(crate) mod shape_delete_contact_channel_input;

pub(crate) mod shape_delete_contact_input;

pub(crate) mod shape_delete_rotation_input;

pub(crate) mod shape_delete_rotation_override_input;

pub(crate) mod shape_describe_engagement_input;

pub(crate) mod shape_describe_page_input;

pub(crate) mod shape_get_contact_channel_input;

pub(crate) mod shape_get_contact_input;

pub(crate) mod shape_get_contact_policy_input;

pub(crate) mod shape_get_rotation_input;

pub(crate) mod shape_get_rotation_override_input;

pub(crate) mod shape_internal_server_exception;

pub(crate) mod shape_list_contact_channels_input;

pub(crate) mod shape_list_contacts_input;

pub(crate) mod shape_list_engagements_input;

pub(crate) mod shape_list_page_receipts_input;

pub(crate) mod shape_list_page_resolutions_input;

pub(crate) mod shape_list_pages_by_contact_input;

pub(crate) mod shape_list_pages_by_engagement_input;

pub(crate) mod shape_list_preview_rotation_shifts_input;

pub(crate) mod shape_list_rotation_overrides_input;

pub(crate) mod shape_list_rotation_shifts_input;

pub(crate) mod shape_list_rotations_input;

pub(crate) mod shape_list_tags_for_resource_input;

pub(crate) mod shape_put_contact_policy_input;

pub(crate) mod shape_resource_not_found_exception;

pub(crate) mod shape_send_activation_code_input;

pub(crate) mod shape_service_quota_exceeded_exception;

pub(crate) mod shape_start_engagement_input;

pub(crate) mod shape_stop_engagement_input;

pub(crate) mod shape_tag_resource_input;

pub(crate) mod shape_throttling_exception;

pub(crate) mod shape_untag_resource_input;

pub(crate) mod shape_update_contact_channel_input;

pub(crate) mod shape_update_contact_input;

pub(crate) mod shape_update_rotation_input;

pub(crate) mod shape_validation_exception;

pub(crate) mod shape_contact_channel_address;

pub(crate) mod shape_contact_channel_list;

pub(crate) mod shape_contacts_list;

pub(crate) mod shape_dependent_entity_list;

pub(crate) mod shape_engagements_list;

pub(crate) mod shape_pages_list;

pub(crate) mod shape_plan;

pub(crate) mod shape_preview_override;

pub(crate) mod shape_receipts_list;

pub(crate) mod shape_recurrence_settings;

pub(crate) mod shape_resolution_list;

pub(crate) mod shape_rotation_contacts_arn_list;

pub(crate) mod shape_rotation_overrides;

pub(crate) mod shape_rotation_shifts;

pub(crate) mod shape_rotations;

pub(crate) mod shape_ssm_contacts_arn_list;

pub(crate) mod shape_tag;

pub(crate) mod shape_tags_list;

pub(crate) mod shape_time_range;

pub(crate) mod shape_validation_exception_field_list;

pub(crate) mod shape_contact;

pub(crate) mod shape_contact_channel;

pub(crate) mod shape_coverage_time;

pub(crate) mod shape_daily_settings;

pub(crate) mod shape_dependent_entity;

pub(crate) mod shape_engagement;

pub(crate) mod shape_hand_off_time;

pub(crate) mod shape_monthly_setting;

pub(crate) mod shape_monthly_settings;

pub(crate) mod shape_page;

pub(crate) mod shape_receipt;

pub(crate) mod shape_resolution_contact;

pub(crate) mod shape_rotation;

pub(crate) mod shape_rotation_override;

pub(crate) mod shape_rotation_shift;

pub(crate) mod shape_shift_coverages_map;

pub(crate) mod shape_stage;

pub(crate) mod shape_stages_list;

pub(crate) mod shape_validation_exception_field;

pub(crate) mod shape_weekly_setting;

pub(crate) mod shape_weekly_settings;

pub(crate) mod shape_coverage_times;

pub(crate) mod shape_shift_details;

pub(crate) mod shape_target;

pub(crate) mod shape_channel_target_info;

pub(crate) mod shape_contact_target_info;

pub(crate) mod shape_targets_list;

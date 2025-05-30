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

pub(crate) mod shape_add_tags_to_resource;

pub(crate) mod shape_associate_ops_item_related_item;

pub(crate) mod shape_cancel_command;

pub(crate) mod shape_cancel_maintenance_window_execution;

pub(crate) mod shape_create_activation;

pub(crate) mod shape_create_association;

pub(crate) mod shape_create_association_batch;

pub(crate) mod shape_create_document;

pub(crate) mod shape_create_maintenance_window;

pub(crate) mod shape_create_ops_item;

pub(crate) mod shape_create_ops_metadata;

pub(crate) mod shape_create_patch_baseline;

pub(crate) mod shape_create_resource_data_sync;

pub(crate) mod shape_delete_activation;

pub(crate) mod shape_delete_association;

pub(crate) mod shape_delete_document;

pub(crate) mod shape_delete_inventory;

pub(crate) mod shape_delete_maintenance_window;

pub(crate) mod shape_delete_ops_item;

pub(crate) mod shape_delete_ops_metadata;

pub(crate) mod shape_delete_parameter;

pub(crate) mod shape_delete_parameters;

pub(crate) mod shape_delete_patch_baseline;

pub(crate) mod shape_delete_resource_data_sync;

pub(crate) mod shape_delete_resource_policy;

pub(crate) mod shape_deregister_managed_instance;

pub(crate) mod shape_deregister_patch_baseline_for_patch_group;

pub(crate) mod shape_deregister_target_from_maintenance_window;

pub(crate) mod shape_deregister_task_from_maintenance_window;

pub(crate) mod shape_describe_activations;

pub(crate) mod shape_describe_association;

pub(crate) mod shape_describe_association_execution_targets;

pub(crate) mod shape_describe_association_executions;

pub(crate) mod shape_describe_automation_executions;

pub(crate) mod shape_describe_automation_step_executions;

pub(crate) mod shape_describe_available_patches;

pub(crate) mod shape_describe_document;

pub(crate) mod shape_describe_document_permission;

pub(crate) mod shape_describe_effective_instance_associations;

pub(crate) mod shape_describe_effective_patches_for_patch_baseline;

pub(crate) mod shape_describe_instance_associations_status;

pub(crate) mod shape_describe_instance_information;

pub(crate) mod shape_describe_instance_patch_states;

pub(crate) mod shape_describe_instance_patch_states_for_patch_group;

pub(crate) mod shape_describe_instance_patches;

pub(crate) mod shape_describe_instance_properties;

pub(crate) mod shape_describe_inventory_deletions;

pub(crate) mod shape_describe_maintenance_window_execution_task_invocations;

pub(crate) mod shape_describe_maintenance_window_execution_tasks;

pub(crate) mod shape_describe_maintenance_window_executions;

pub(crate) mod shape_describe_maintenance_window_schedule;

pub(crate) mod shape_describe_maintenance_window_targets;

pub(crate) mod shape_describe_maintenance_window_tasks;

pub(crate) mod shape_describe_maintenance_windows;

pub(crate) mod shape_describe_maintenance_windows_for_target;

pub(crate) mod shape_describe_ops_items;

pub(crate) mod shape_describe_parameters;

pub(crate) mod shape_describe_patch_baselines;

pub(crate) mod shape_describe_patch_group_state;

pub(crate) mod shape_describe_patch_groups;

pub(crate) mod shape_describe_patch_properties;

pub(crate) mod shape_describe_sessions;

pub(crate) mod shape_disassociate_ops_item_related_item;

pub(crate) mod shape_get_access_token;

pub(crate) mod shape_get_automation_execution;

pub(crate) mod shape_get_calendar_state;

pub(crate) mod shape_get_command_invocation;

pub(crate) mod shape_get_connection_status;

pub(crate) mod shape_get_default_patch_baseline;

pub(crate) mod shape_get_deployable_patch_snapshot_for_instance;

pub(crate) mod shape_get_document;

pub(crate) mod shape_get_execution_preview;

pub(crate) mod shape_get_inventory;

pub(crate) mod shape_get_inventory_schema;

pub(crate) mod shape_get_maintenance_window;

pub(crate) mod shape_get_maintenance_window_execution;

pub(crate) mod shape_get_maintenance_window_execution_task;

pub(crate) mod shape_get_maintenance_window_execution_task_invocation;

pub(crate) mod shape_get_maintenance_window_task;

pub(crate) mod shape_get_ops_item;

pub(crate) mod shape_get_ops_metadata;

pub(crate) mod shape_get_ops_summary;

pub(crate) mod shape_get_parameter;

pub(crate) mod shape_get_parameter_history;

pub(crate) mod shape_get_parameters;

pub(crate) mod shape_get_parameters_by_path;

pub(crate) mod shape_get_patch_baseline;

pub(crate) mod shape_get_patch_baseline_for_patch_group;

pub(crate) mod shape_get_resource_policies;

pub(crate) mod shape_get_service_setting;

pub(crate) mod shape_label_parameter_version;

pub(crate) mod shape_list_association_versions;

pub(crate) mod shape_list_associations;

pub(crate) mod shape_list_command_invocations;

pub(crate) mod shape_list_commands;

pub(crate) mod shape_list_compliance_items;

pub(crate) mod shape_list_compliance_summaries;

pub(crate) mod shape_list_document_metadata_history;

pub(crate) mod shape_list_document_versions;

pub(crate) mod shape_list_documents;

pub(crate) mod shape_list_inventory_entries;

pub(crate) mod shape_list_nodes;

pub(crate) mod shape_list_nodes_summary;

pub(crate) mod shape_list_ops_item_events;

pub(crate) mod shape_list_ops_item_related_items;

pub(crate) mod shape_list_ops_metadata;

pub(crate) mod shape_list_resource_compliance_summaries;

pub(crate) mod shape_list_resource_data_sync;

pub(crate) mod shape_list_tags_for_resource;

pub(crate) mod shape_modify_document_permission;

pub(crate) mod shape_put_compliance_items;

pub(crate) mod shape_put_inventory;

pub(crate) mod shape_put_parameter;

pub(crate) mod shape_put_resource_policy;

pub(crate) mod shape_register_default_patch_baseline;

pub(crate) mod shape_register_patch_baseline_for_patch_group;

pub(crate) mod shape_register_target_with_maintenance_window;

pub(crate) mod shape_register_task_with_maintenance_window;

pub(crate) mod shape_remove_tags_from_resource;

pub(crate) mod shape_reset_service_setting;

pub(crate) mod shape_resume_session;

pub(crate) mod shape_send_automation_signal;

pub(crate) mod shape_send_command;

pub(crate) mod shape_start_access_request;

pub(crate) mod shape_start_associations_once;

pub(crate) mod shape_start_automation_execution;

pub(crate) mod shape_start_change_request_execution;

pub(crate) mod shape_start_execution_preview;

pub(crate) mod shape_start_session;

pub(crate) mod shape_stop_automation_execution;

pub(crate) mod shape_terminate_session;

pub(crate) mod shape_unlabel_parameter_version;

pub(crate) mod shape_update_association;

pub(crate) mod shape_update_association_status;

pub(crate) mod shape_update_document;

pub(crate) mod shape_update_document_default_version;

pub(crate) mod shape_update_document_metadata;

pub(crate) mod shape_update_maintenance_window;

pub(crate) mod shape_update_maintenance_window_target;

pub(crate) mod shape_update_maintenance_window_task;

pub(crate) mod shape_update_managed_instance_role;

pub(crate) mod shape_update_ops_item;

pub(crate) mod shape_update_ops_metadata;

pub(crate) mod shape_update_patch_baseline;

pub(crate) mod shape_update_resource_data_sync;

pub(crate) mod shape_update_service_setting;

pub(crate) fn or_empty_doc(data: &[u8]) -> &[u8] {
    if data.is_empty() {
        b"{}"
    } else {
        data
    }
}

pub(crate) mod shape_access_denied_exception;

pub(crate) mod shape_add_tags_to_resource_input;

pub(crate) mod shape_already_exists_exception;

pub(crate) mod shape_associate_ops_item_related_item_input;

pub(crate) mod shape_associated_instances;

pub(crate) mod shape_association_already_exists;

pub(crate) mod shape_association_does_not_exist;

pub(crate) mod shape_association_execution_does_not_exist;

pub(crate) mod shape_association_limit_exceeded;

pub(crate) mod shape_association_version_limit_exceeded;

pub(crate) mod shape_automation_definition_not_approved_exception;

pub(crate) mod shape_automation_definition_not_found_exception;

pub(crate) mod shape_automation_definition_version_not_found_exception;

pub(crate) mod shape_automation_execution_limit_exceeded_exception;

pub(crate) mod shape_automation_execution_not_found_exception;

pub(crate) mod shape_automation_step_not_found_exception;

pub(crate) mod shape_cancel_command_input;

pub(crate) mod shape_cancel_maintenance_window_execution_input;

pub(crate) mod shape_compliance_type_count_limit_exceeded_exception;

pub(crate) mod shape_create_activation_input;

pub(crate) mod shape_create_association_batch_input;

pub(crate) mod shape_create_association_input;

pub(crate) mod shape_create_document_input;

pub(crate) mod shape_create_maintenance_window_input;

pub(crate) mod shape_create_ops_item_input;

pub(crate) mod shape_create_ops_metadata_input;

pub(crate) mod shape_create_patch_baseline_input;

pub(crate) mod shape_create_resource_data_sync_input;

pub(crate) mod shape_custom_schema_count_limit_exceeded_exception;

pub(crate) mod shape_delete_activation_input;

pub(crate) mod shape_delete_association_input;

pub(crate) mod shape_delete_document_input;

pub(crate) mod shape_delete_inventory_input;

pub(crate) mod shape_delete_maintenance_window_input;

pub(crate) mod shape_delete_ops_item_input;

pub(crate) mod shape_delete_ops_metadata_input;

pub(crate) mod shape_delete_parameter_input;

pub(crate) mod shape_delete_parameters_input;

pub(crate) mod shape_delete_patch_baseline_input;

pub(crate) mod shape_delete_resource_data_sync_input;

pub(crate) mod shape_delete_resource_policy_input;

pub(crate) mod shape_deregister_managed_instance_input;

pub(crate) mod shape_deregister_patch_baseline_for_patch_group_input;

pub(crate) mod shape_deregister_target_from_maintenance_window_input;

pub(crate) mod shape_deregister_task_from_maintenance_window_input;

pub(crate) mod shape_describe_activations_input;

pub(crate) mod shape_describe_association_execution_targets_input;

pub(crate) mod shape_describe_association_executions_input;

pub(crate) mod shape_describe_association_input;

pub(crate) mod shape_describe_automation_executions_input;

pub(crate) mod shape_describe_automation_step_executions_input;

pub(crate) mod shape_describe_available_patches_input;

pub(crate) mod shape_describe_document_input;

pub(crate) mod shape_describe_document_permission_input;

pub(crate) mod shape_describe_effective_instance_associations_input;

pub(crate) mod shape_describe_effective_patches_for_patch_baseline_input;

pub(crate) mod shape_describe_instance_associations_status_input;

pub(crate) mod shape_describe_instance_information_input;

pub(crate) mod shape_describe_instance_patch_states_for_patch_group_input;

pub(crate) mod shape_describe_instance_patch_states_input;

pub(crate) mod shape_describe_instance_patches_input;

pub(crate) mod shape_describe_instance_properties_input;

pub(crate) mod shape_describe_inventory_deletions_input;

pub(crate) mod shape_describe_maintenance_window_execution_task_invocations_input;

pub(crate) mod shape_describe_maintenance_window_execution_tasks_input;

pub(crate) mod shape_describe_maintenance_window_executions_input;

pub(crate) mod shape_describe_maintenance_window_schedule_input;

pub(crate) mod shape_describe_maintenance_window_targets_input;

pub(crate) mod shape_describe_maintenance_window_tasks_input;

pub(crate) mod shape_describe_maintenance_windows_for_target_input;

pub(crate) mod shape_describe_maintenance_windows_input;

pub(crate) mod shape_describe_ops_items_input;

pub(crate) mod shape_describe_parameters_input;

pub(crate) mod shape_describe_patch_baselines_input;

pub(crate) mod shape_describe_patch_group_state_input;

pub(crate) mod shape_describe_patch_groups_input;

pub(crate) mod shape_describe_patch_properties_input;

pub(crate) mod shape_describe_sessions_input;

pub(crate) mod shape_disassociate_ops_item_related_item_input;

pub(crate) mod shape_document_already_exists;

pub(crate) mod shape_document_limit_exceeded;

pub(crate) mod shape_document_permission_limit;

pub(crate) mod shape_document_version_limit_exceeded;

pub(crate) mod shape_does_not_exist_exception;

pub(crate) mod shape_duplicate_document_content;

pub(crate) mod shape_duplicate_document_version_name;

pub(crate) mod shape_duplicate_instance_id;

pub(crate) mod shape_feature_not_available_exception;

pub(crate) mod shape_get_access_token_input;

pub(crate) mod shape_get_automation_execution_input;

pub(crate) mod shape_get_calendar_state_input;

pub(crate) mod shape_get_command_invocation_input;

pub(crate) mod shape_get_connection_status_input;

pub(crate) mod shape_get_default_patch_baseline_input;

pub(crate) mod shape_get_deployable_patch_snapshot_for_instance_input;

pub(crate) mod shape_get_document_input;

pub(crate) mod shape_get_execution_preview_input;

pub(crate) mod shape_get_inventory_input;

pub(crate) mod shape_get_inventory_schema_input;

pub(crate) mod shape_get_maintenance_window_execution_input;

pub(crate) mod shape_get_maintenance_window_execution_task_input;

pub(crate) mod shape_get_maintenance_window_execution_task_invocation_input;

pub(crate) mod shape_get_maintenance_window_input;

pub(crate) mod shape_get_maintenance_window_task_input;

pub(crate) mod shape_get_ops_item_input;

pub(crate) mod shape_get_ops_metadata_input;

pub(crate) mod shape_get_ops_summary_input;

pub(crate) mod shape_get_parameter_history_input;

pub(crate) mod shape_get_parameter_input;

pub(crate) mod shape_get_parameters_by_path_input;

pub(crate) mod shape_get_parameters_input;

pub(crate) mod shape_get_patch_baseline_for_patch_group_input;

pub(crate) mod shape_get_patch_baseline_input;

pub(crate) mod shape_get_resource_policies_input;

pub(crate) mod shape_get_service_setting_input;

pub(crate) mod shape_hierarchy_level_limit_exceeded_exception;

pub(crate) mod shape_hierarchy_type_mismatch_exception;

pub(crate) mod shape_idempotent_parameter_mismatch;

pub(crate) mod shape_incompatible_policy_exception;

pub(crate) mod shape_internal_server_error;

pub(crate) mod shape_invalid_activation;

pub(crate) mod shape_invalid_activation_id;

pub(crate) mod shape_invalid_aggregator_exception;

pub(crate) mod shape_invalid_allowed_pattern_exception;

pub(crate) mod shape_invalid_association;

pub(crate) mod shape_invalid_association_version;

pub(crate) mod shape_invalid_automation_execution_parameters_exception;

pub(crate) mod shape_invalid_automation_signal_exception;

pub(crate) mod shape_invalid_automation_status_update_exception;

pub(crate) mod shape_invalid_command_id;

pub(crate) mod shape_invalid_delete_inventory_parameters_exception;

pub(crate) mod shape_invalid_deletion_id_exception;

pub(crate) mod shape_invalid_document;

pub(crate) mod shape_invalid_document_content;

pub(crate) mod shape_invalid_document_operation;

pub(crate) mod shape_invalid_document_schema_version;

pub(crate) mod shape_invalid_document_type;

pub(crate) mod shape_invalid_document_version;

pub(crate) mod shape_invalid_filter;

pub(crate) mod shape_invalid_filter_key;

pub(crate) mod shape_invalid_filter_option;

pub(crate) mod shape_invalid_filter_value;

pub(crate) mod shape_invalid_instance_id;

pub(crate) mod shape_invalid_instance_information_filter_value;

pub(crate) mod shape_invalid_instance_property_filter_value;

pub(crate) mod shape_invalid_inventory_group_exception;

pub(crate) mod shape_invalid_inventory_item_context_exception;

pub(crate) mod shape_invalid_inventory_request_exception;

pub(crate) mod shape_invalid_item_content_exception;

pub(crate) mod shape_invalid_key_id;

pub(crate) mod shape_invalid_next_token;

pub(crate) mod shape_invalid_notification_config;

pub(crate) mod shape_invalid_option_exception;

pub(crate) mod shape_invalid_output_folder;

pub(crate) mod shape_invalid_output_location;

pub(crate) mod shape_invalid_parameters;

pub(crate) mod shape_invalid_permission_type;

pub(crate) mod shape_invalid_plugin_name;

pub(crate) mod shape_invalid_policy_attribute_exception;

pub(crate) mod shape_invalid_policy_type_exception;

pub(crate) mod shape_invalid_resource_id;

pub(crate) mod shape_invalid_resource_type;

pub(crate) mod shape_invalid_result_attribute_exception;

pub(crate) mod shape_invalid_role;

pub(crate) mod shape_invalid_schedule;

pub(crate) mod shape_invalid_tag;

pub(crate) mod shape_invalid_target;

pub(crate) mod shape_invalid_target_maps;

pub(crate) mod shape_invalid_type_name_exception;

pub(crate) mod shape_invalid_update;

pub(crate) mod shape_invocation_does_not_exist;

pub(crate) mod shape_item_content_mismatch_exception;

pub(crate) mod shape_item_size_limit_exceeded_exception;

pub(crate) mod shape_label_parameter_version_input;

pub(crate) mod shape_list_association_versions_input;

pub(crate) mod shape_list_associations_input;

pub(crate) mod shape_list_command_invocations_input;

pub(crate) mod shape_list_commands_input;

pub(crate) mod shape_list_compliance_items_input;

pub(crate) mod shape_list_compliance_summaries_input;

pub(crate) mod shape_list_document_metadata_history_input;

pub(crate) mod shape_list_document_versions_input;

pub(crate) mod shape_list_documents_input;

pub(crate) mod shape_list_inventory_entries_input;

pub(crate) mod shape_list_nodes_input;

pub(crate) mod shape_list_nodes_summary_input;

pub(crate) mod shape_list_ops_item_events_input;

pub(crate) mod shape_list_ops_item_related_items_input;

pub(crate) mod shape_list_ops_metadata_input;

pub(crate) mod shape_list_resource_compliance_summaries_input;

pub(crate) mod shape_list_resource_data_sync_input;

pub(crate) mod shape_list_tags_for_resource_input;

pub(crate) mod shape_malformed_resource_policy_document_exception;

pub(crate) mod shape_max_document_size_exceeded;

pub(crate) mod shape_modify_document_permission_input;

pub(crate) mod shape_ops_item_access_denied_exception;

pub(crate) mod shape_ops_item_already_exists_exception;

pub(crate) mod shape_ops_item_conflict_exception;

pub(crate) mod shape_ops_item_invalid_parameter_exception;

pub(crate) mod shape_ops_item_limit_exceeded_exception;

pub(crate) mod shape_ops_item_not_found_exception;

pub(crate) mod shape_ops_item_related_item_already_exists_exception;

pub(crate) mod shape_ops_item_related_item_association_not_found_exception;

pub(crate) mod shape_ops_metadata_already_exists_exception;

pub(crate) mod shape_ops_metadata_invalid_argument_exception;

pub(crate) mod shape_ops_metadata_key_limit_exceeded_exception;

pub(crate) mod shape_ops_metadata_limit_exceeded_exception;

pub(crate) mod shape_ops_metadata_not_found_exception;

pub(crate) mod shape_ops_metadata_too_many_updates_exception;

pub(crate) mod shape_parameter_already_exists;

pub(crate) mod shape_parameter_limit_exceeded;

pub(crate) mod shape_parameter_max_version_limit_exceeded;

pub(crate) mod shape_parameter_not_found;

pub(crate) mod shape_parameter_pattern_mismatch_exception;

pub(crate) mod shape_parameter_version_label_limit_exceeded;

pub(crate) mod shape_parameter_version_not_found;

pub(crate) mod shape_policies_limit_exceeded_exception;

pub(crate) mod shape_put_compliance_items_input;

pub(crate) mod shape_put_inventory_input;

pub(crate) mod shape_put_parameter_input;

pub(crate) mod shape_put_resource_policy_input;

pub(crate) mod shape_register_default_patch_baseline_input;

pub(crate) mod shape_register_patch_baseline_for_patch_group_input;

pub(crate) mod shape_register_target_with_maintenance_window_input;

pub(crate) mod shape_register_task_with_maintenance_window_input;

pub(crate) mod shape_remove_tags_from_resource_input;

pub(crate) mod shape_reset_service_setting_input;

pub(crate) mod shape_resource_data_sync_already_exists_exception;

pub(crate) mod shape_resource_data_sync_conflict_exception;

pub(crate) mod shape_resource_data_sync_count_exceeded_exception;

pub(crate) mod shape_resource_data_sync_invalid_configuration_exception;

pub(crate) mod shape_resource_data_sync_not_found_exception;

pub(crate) mod shape_resource_in_use_exception;

pub(crate) mod shape_resource_limit_exceeded_exception;

pub(crate) mod shape_resource_not_found_exception;

pub(crate) mod shape_resource_policy_conflict_exception;

pub(crate) mod shape_resource_policy_invalid_parameter_exception;

pub(crate) mod shape_resource_policy_limit_exceeded_exception;

pub(crate) mod shape_resource_policy_not_found_exception;

pub(crate) mod shape_resume_session_input;

pub(crate) mod shape_send_automation_signal_input;

pub(crate) mod shape_send_command_input;

pub(crate) mod shape_service_quota_exceeded_exception;

pub(crate) mod shape_service_setting_not_found;

pub(crate) mod shape_start_access_request_input;

pub(crate) mod shape_start_associations_once_input;

pub(crate) mod shape_start_automation_execution_input;

pub(crate) mod shape_start_change_request_execution_input;

pub(crate) mod shape_start_execution_preview_input;

pub(crate) mod shape_start_session_input;

pub(crate) mod shape_status_unchanged;

pub(crate) mod shape_stop_automation_execution_input;

pub(crate) mod shape_sub_type_count_limit_exceeded_exception;

pub(crate) mod shape_target_in_use_exception;

pub(crate) mod shape_target_not_connected;

pub(crate) mod shape_terminate_session_input;

pub(crate) mod shape_throttling_exception;

pub(crate) mod shape_too_many_tags_error;

pub(crate) mod shape_too_many_updates;

pub(crate) mod shape_total_size_limit_exceeded_exception;

pub(crate) mod shape_unlabel_parameter_version_input;

pub(crate) mod shape_unsupported_calendar_exception;

pub(crate) mod shape_unsupported_feature_required_exception;

pub(crate) mod shape_unsupported_inventory_item_context_exception;

pub(crate) mod shape_unsupported_inventory_schema_version_exception;

pub(crate) mod shape_unsupported_operating_system;

pub(crate) mod shape_unsupported_operation_exception;

pub(crate) mod shape_unsupported_parameter_type;

pub(crate) mod shape_unsupported_platform_type;

pub(crate) mod shape_update_association_input;

pub(crate) mod shape_update_association_status_input;

pub(crate) mod shape_update_document_default_version_input;

pub(crate) mod shape_update_document_input;

pub(crate) mod shape_update_document_metadata_input;

pub(crate) mod shape_update_maintenance_window_input;

pub(crate) mod shape_update_maintenance_window_target_input;

pub(crate) mod shape_update_maintenance_window_task_input;

pub(crate) mod shape_update_managed_instance_role_input;

pub(crate) mod shape_update_ops_item_input;

pub(crate) mod shape_update_ops_metadata_input;

pub(crate) mod shape_update_patch_baseline_input;

pub(crate) mod shape_update_resource_data_sync_input;

pub(crate) mod shape_update_service_setting_input;

pub(crate) mod shape_validation_exception;

pub(crate) mod shape_account_id_list;

pub(crate) mod shape_account_sharing_info_list;

pub(crate) mod shape_activation_list;

pub(crate) mod shape_alarm_configuration;

pub(crate) mod shape_alarm_state_information_list;

pub(crate) mod shape_association_description;

pub(crate) mod shape_association_description_list;

pub(crate) mod shape_association_execution_filter;

pub(crate) mod shape_association_execution_targets_filter;

pub(crate) mod shape_association_execution_targets_list;

pub(crate) mod shape_association_executions_list;

pub(crate) mod shape_association_filter;

pub(crate) mod shape_association_list;

pub(crate) mod shape_association_status;

pub(crate) mod shape_association_version_list;

pub(crate) mod shape_attachment_content_list;

pub(crate) mod shape_attachments_source;

pub(crate) mod shape_automation_execution;

pub(crate) mod shape_automation_execution_filter;

pub(crate) mod shape_automation_execution_metadata_list;

pub(crate) mod shape_baseline_override;

pub(crate) mod shape_cloud_watch_output_config;

pub(crate) mod shape_command;

pub(crate) mod shape_command_filter;

pub(crate) mod shape_command_invocation_list;

pub(crate) mod shape_command_list;

pub(crate) mod shape_compliance_execution_summary;

pub(crate) mod shape_compliance_item_entry;

pub(crate) mod shape_compliance_item_list;

pub(crate) mod shape_compliance_string_filter;

pub(crate) mod shape_compliance_summary_item_list;

pub(crate) mod shape_create_association_batch_request_entry;

pub(crate) mod shape_credentials;

pub(crate) mod shape_describe_activations_filter;

pub(crate) mod shape_document_default_version_description;

pub(crate) mod shape_document_description;

pub(crate) mod shape_document_filter;

pub(crate) mod shape_document_identifier_list;

pub(crate) mod shape_document_key_values_filter;

pub(crate) mod shape_document_metadata_response_info;

pub(crate) mod shape_document_requires;

pub(crate) mod shape_document_requires_list;

pub(crate) mod shape_document_reviews;

pub(crate) mod shape_document_version_list;

pub(crate) mod shape_effective_patch_list;

pub(crate) mod shape_execution_inputs;

pub(crate) mod shape_execution_preview;

pub(crate) mod shape_failed_create_association_list;

pub(crate) mod shape_get_resource_policies_response_entries;

pub(crate) mod shape_instance_association_list;

pub(crate) mod shape_instance_association_output_location;

pub(crate) mod shape_instance_association_status_infos;

pub(crate) mod shape_instance_information_filter;

pub(crate) mod shape_instance_information_list;

pub(crate) mod shape_instance_information_string_filter;

pub(crate) mod shape_instance_patch_state_filter;

pub(crate) mod shape_instance_patch_state_list;

pub(crate) mod shape_instance_patch_states_list;

pub(crate) mod shape_instance_properties;

pub(crate) mod shape_instance_property_filter;

pub(crate) mod shape_instance_property_string_filter;

pub(crate) mod shape_inventory_aggregator;

pub(crate) mod shape_inventory_deletion_summary;

pub(crate) mod shape_inventory_deletions_list;

pub(crate) mod shape_inventory_filter;

pub(crate) mod shape_inventory_item;

pub(crate) mod shape_inventory_item_entry_list;

pub(crate) mod shape_inventory_item_schema_result_list;

pub(crate) mod shape_inventory_result_entity_list;

pub(crate) mod shape_logging_info;

pub(crate) mod shape_maintenance_window_execution_list;

pub(crate) mod shape_maintenance_window_execution_task_id_list;

pub(crate) mod shape_maintenance_window_execution_task_identity_list;

pub(crate) mod shape_maintenance_window_execution_task_invocation_identity_list;

pub(crate) mod shape_maintenance_window_filter;

pub(crate) mod shape_maintenance_window_identity_list;

pub(crate) mod shape_maintenance_window_target_list;

pub(crate) mod shape_maintenance_window_task_invocation_parameters;

pub(crate) mod shape_maintenance_window_task_list;

pub(crate) mod shape_maintenance_window_task_parameter_value_expression;

pub(crate) mod shape_maintenance_window_task_parameters;

pub(crate) mod shape_maintenance_window_task_parameters_list;

pub(crate) mod shape_maintenance_windows_for_target_list;

pub(crate) mod shape_metadata_map;

pub(crate) mod shape_metadata_value;

pub(crate) mod shape_node_aggregator;

pub(crate) mod shape_node_filter;

pub(crate) mod shape_node_list;

pub(crate) mod shape_node_summary_list;

pub(crate) mod shape_notification_config;

pub(crate) mod shape_ops_aggregator;

pub(crate) mod shape_ops_entity_list;

pub(crate) mod shape_ops_filter;

pub(crate) mod shape_ops_item;

pub(crate) mod shape_ops_item_data_value;

pub(crate) mod shape_ops_item_event_filter;

pub(crate) mod shape_ops_item_event_summaries;

pub(crate) mod shape_ops_item_filter;

pub(crate) mod shape_ops_item_notification;

pub(crate) mod shape_ops_item_parameter_names_list;

pub(crate) mod shape_ops_item_related_item_summaries;

pub(crate) mod shape_ops_item_related_items_filter;

pub(crate) mod shape_ops_item_summaries;

pub(crate) mod shape_ops_metadata_filter;

pub(crate) mod shape_ops_metadata_list;

pub(crate) mod shape_ops_result_attribute;

pub(crate) mod shape_parameter;

pub(crate) mod shape_parameter_history_list;

pub(crate) mod shape_parameter_label_list;

pub(crate) mod shape_parameter_list;

pub(crate) mod shape_parameter_metadata_list;

pub(crate) mod shape_parameter_name_list;

pub(crate) mod shape_parameter_string_filter;

pub(crate) mod shape_parameters_filter;

pub(crate) mod shape_patch_baseline_identity_list;

pub(crate) mod shape_patch_compliance_data_list;

pub(crate) mod shape_patch_filter_group;

pub(crate) mod shape_patch_group_list;

pub(crate) mod shape_patch_group_patch_baseline_mapping_list;

pub(crate) mod shape_patch_id_list;

pub(crate) mod shape_patch_list;

pub(crate) mod shape_patch_orchestrator_filter;

pub(crate) mod shape_patch_properties_list;

pub(crate) mod shape_patch_rule_group;

pub(crate) mod shape_patch_source;

pub(crate) mod shape_patch_source_list;

pub(crate) mod shape_registration_metadata_item;

pub(crate) mod shape_related_ops_item;

pub(crate) mod shape_resource_compliance_summary_item_list;

pub(crate) mod shape_resource_data_sync_item_list;

pub(crate) mod shape_resource_data_sync_s3_destination;

pub(crate) mod shape_resource_data_sync_source;

pub(crate) mod shape_resource_policy_parameter_names_list;

pub(crate) mod shape_result_attribute;

pub(crate) mod shape_runbook;

pub(crate) mod shape_scheduled_window_execution_list;

pub(crate) mod shape_service_setting;

pub(crate) mod shape_session_filter;

pub(crate) mod shape_session_list;

pub(crate) mod shape_step_execution_filter;

pub(crate) mod shape_step_execution_list;

pub(crate) mod shape_tag;

pub(crate) mod shape_tag_list;

pub(crate) mod shape_target;

pub(crate) mod shape_target_location;

pub(crate) mod shape_targets;

pub(crate) mod shape_account_sharing_info;

pub(crate) mod shape_activation;

pub(crate) mod shape_alarm;

pub(crate) mod shape_alarm_list;

pub(crate) mod shape_alarm_state_information;

pub(crate) mod shape_association;

pub(crate) mod shape_association_execution;

pub(crate) mod shape_association_execution_target;

pub(crate) mod shape_association_overview;

pub(crate) mod shape_association_version_info;

pub(crate) mod shape_attachment_content;

pub(crate) mod shape_attachment_information_list;

pub(crate) mod shape_automation_execution_inputs;

pub(crate) mod shape_automation_execution_metadata;

pub(crate) mod shape_automation_execution_preview;

pub(crate) mod shape_automation_parameter_map;

pub(crate) mod shape_calendar_name_or_arn_list;

pub(crate) mod shape_category_enum_list;

pub(crate) mod shape_category_list;

pub(crate) mod shape_command_invocation;

pub(crate) mod shape_compliance_item;

pub(crate) mod shape_compliance_summary_item;

pub(crate) mod shape_document_identifier;

pub(crate) mod shape_document_parameter_list;

pub(crate) mod shape_document_review_comment_source;

pub(crate) mod shape_document_reviewer_response_list;

pub(crate) mod shape_document_version_info;

pub(crate) mod shape_effective_patch;

pub(crate) mod shape_failed_create_association;

pub(crate) mod shape_get_resource_policies_response_entry;

pub(crate) mod shape_instance_association;

pub(crate) mod shape_instance_association_status_info;

pub(crate) mod shape_instance_id_list;

pub(crate) mod shape_instance_information;

pub(crate) mod shape_instance_patch_state;

pub(crate) mod shape_instance_property;

pub(crate) mod shape_inventory_deletion_status_item;

pub(crate) mod shape_inventory_deletion_summary_items;

pub(crate) mod shape_inventory_group;

pub(crate) mod shape_inventory_item_entry;

pub(crate) mod shape_inventory_item_schema;

pub(crate) mod shape_inventory_result_entity;

pub(crate) mod shape_maintenance_window_automation_parameters;

pub(crate) mod shape_maintenance_window_execution;

pub(crate) mod shape_maintenance_window_execution_task_identity;

pub(crate) mod shape_maintenance_window_execution_task_invocation_identity;

pub(crate) mod shape_maintenance_window_identity;

pub(crate) mod shape_maintenance_window_identity_for_target;

pub(crate) mod shape_maintenance_window_lambda_parameters;

pub(crate) mod shape_maintenance_window_run_command_parameters;

pub(crate) mod shape_maintenance_window_step_functions_parameters;

pub(crate) mod shape_maintenance_window_target;

pub(crate) mod shape_maintenance_window_task;

pub(crate) mod shape_node;

pub(crate) mod shape_node_summary;

pub(crate) mod shape_ops_entity;

pub(crate) mod shape_ops_item_event_summary;

pub(crate) mod shape_ops_item_notifications;

pub(crate) mod shape_ops_item_operational_data;

pub(crate) mod shape_ops_item_related_item_summary;

pub(crate) mod shape_ops_item_summary;

pub(crate) mod shape_ops_metadata;

pub(crate) mod shape_parameter_history;

pub(crate) mod shape_parameter_metadata;

pub(crate) mod shape_parameters;

pub(crate) mod shape_patch;

pub(crate) mod shape_patch_baseline_identity;

pub(crate) mod shape_patch_compliance_data;

pub(crate) mod shape_patch_filter;

pub(crate) mod shape_patch_filter_list;

pub(crate) mod shape_patch_group_patch_baseline_mapping;

pub(crate) mod shape_patch_property_entry;

pub(crate) mod shape_patch_rule;

pub(crate) mod shape_patch_rule_list;

pub(crate) mod shape_platform_type_list;

pub(crate) mod shape_progress_counters;

pub(crate) mod shape_related_ops_items;

pub(crate) mod shape_resolved_targets;

pub(crate) mod shape_resource_compliance_summary_item;

pub(crate) mod shape_resource_data_sync_aws_organizations_source;

pub(crate) mod shape_resource_data_sync_destination_data_sharing;

pub(crate) mod shape_resource_data_sync_item;

pub(crate) mod shape_review_information_list;

pub(crate) mod shape_runbooks;

pub(crate) mod shape_s3_output_location;

pub(crate) mod shape_scheduled_window_execution;

pub(crate) mod shape_session;

pub(crate) mod shape_step_execution;

pub(crate) mod shape_target_locations;

pub(crate) mod shape_target_maps;

pub(crate) mod shape_association_status_aggregated_count;

pub(crate) mod shape_attachment_information;

pub(crate) mod shape_automation_parameter_value_list;

pub(crate) mod shape_command_plugin_list;

pub(crate) mod shape_compliance_item_details;

pub(crate) mod shape_compliant_summary;

pub(crate) mod shape_document_parameter;

pub(crate) mod shape_document_reviewer_response_source;

pub(crate) mod shape_failure_details;

pub(crate) mod shape_instance_aggregated_association_overview;

pub(crate) mod shape_instance_association_output_url;

pub(crate) mod shape_inventory_deletion_summary_item;

pub(crate) mod shape_inventory_item_attribute_list;

pub(crate) mod shape_inventory_result_item_map;

pub(crate) mod shape_maintenance_window_task_parameter_value_list;

pub(crate) mod shape_node_owner_info;

pub(crate) mod shape_node_type;

pub(crate) mod shape_non_compliant_summary;

pub(crate) mod shape_normal_string_map;

pub(crate) mod shape_notification_event_list;

pub(crate) mod shape_ops_entity_item_map;

pub(crate) mod shape_ops_item_identity;

pub(crate) mod shape_output_source;

pub(crate) mod shape_parameter_policy_list;

pub(crate) mod shape_parameter_value_list;

pub(crate) mod shape_parent_step_details;

pub(crate) mod shape_patch_advisory_id_list;

pub(crate) mod shape_patch_bugzilla_id_list;

pub(crate) mod shape_patch_cve_id_list;

pub(crate) mod shape_patch_source_product_list;

pub(crate) mod shape_patch_status;

pub(crate) mod shape_region_list;

pub(crate) mod shape_resource_data_sync_organizational_unit;

pub(crate) mod shape_resource_data_sync_source_with_state;

pub(crate) mod shape_review_information;

pub(crate) mod shape_session_manager_output_url;

pub(crate) mod shape_step_preview_map;

pub(crate) mod shape_target_map;

pub(crate) mod shape_target_parameter_list;

pub(crate) mod shape_target_preview_list;

pub(crate) mod shape_target_values;

pub(crate) mod shape_valid_next_step_list;

pub(crate) mod shape_accounts;

pub(crate) mod shape_command_plugin;

pub(crate) mod shape_document_review_comment_list;

pub(crate) mod shape_exclude_accounts;

pub(crate) mod shape_instance_association_status_aggregated_count;

pub(crate) mod shape_instance_info;

pub(crate) mod shape_inventory_item_attribute;

pub(crate) mod shape_inventory_result_item;

pub(crate) mod shape_ops_entity_item;

pub(crate) mod shape_parameter_inline_policy;

pub(crate) mod shape_patch_filter_value_list;

pub(crate) mod shape_regions;

pub(crate) mod shape_resource_data_sync_source_region_list;

pub(crate) mod shape_s3_output_url;

pub(crate) mod shape_severity_summary;

pub(crate) mod shape_target_map_value_list;

pub(crate) mod shape_target_preview;

pub(crate) mod shape_ops_entity_item_entry_list;

pub(crate) mod shape_resource_data_sync_organizational_unit_list;

pub(crate) mod shape_ops_entity_item_entry;

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

pub(crate) mod shape_accept_engagement_invitation;

pub(crate) mod shape_assign_opportunity;

pub(crate) mod shape_associate_opportunity;

pub(crate) mod shape_create_engagement;

pub(crate) mod shape_create_engagement_invitation;

pub(crate) mod shape_create_opportunity;

pub(crate) mod shape_create_resource_snapshot;

pub(crate) mod shape_create_resource_snapshot_job;

pub(crate) mod shape_delete_resource_snapshot_job;

pub(crate) mod shape_disassociate_opportunity;

pub(crate) mod shape_get_aws_opportunity_summary;

pub(crate) mod shape_get_engagement;

pub(crate) mod shape_get_engagement_invitation;

pub(crate) mod shape_get_opportunity;

pub(crate) mod shape_get_resource_snapshot;

pub(crate) mod shape_get_resource_snapshot_job;

pub(crate) mod shape_get_selling_system_settings;

pub(crate) mod shape_list_engagement_by_accepting_invitation_tasks;

pub(crate) mod shape_list_engagement_from_opportunity_tasks;

pub(crate) mod shape_list_engagement_invitations;

pub(crate) mod shape_list_engagement_members;

pub(crate) mod shape_list_engagement_resource_associations;

pub(crate) mod shape_list_engagements;

pub(crate) mod shape_list_opportunities;

pub(crate) mod shape_list_resource_snapshot_jobs;

pub(crate) mod shape_list_resource_snapshots;

pub(crate) mod shape_list_solutions;

pub(crate) mod shape_list_tags_for_resource;

pub(crate) mod shape_put_selling_system_settings;

pub(crate) mod shape_reject_engagement_invitation;

pub(crate) mod shape_start_engagement_by_accepting_invitation_task;

pub(crate) mod shape_start_engagement_from_opportunity_task;

pub(crate) mod shape_start_resource_snapshot_job;

pub(crate) mod shape_stop_resource_snapshot_job;

pub(crate) mod shape_submit_opportunity;

pub(crate) mod shape_tag_resource;

pub(crate) mod shape_untag_resource;

pub(crate) mod shape_update_opportunity;

pub(crate) mod shape_accept_engagement_invitation_input;

pub(crate) fn or_empty_doc(data: &[u8]) -> &[u8] {
    if data.is_empty() {
        b"{}"
    } else {
        data
    }
}

pub(crate) mod shape_access_denied_exception;

pub(crate) mod shape_assign_opportunity_input;

pub(crate) mod shape_associate_opportunity_input;

pub(crate) mod shape_conflict_exception;

pub(crate) mod shape_create_engagement_input;

pub(crate) mod shape_create_engagement_invitation_input;

pub(crate) mod shape_create_opportunity_input;

pub(crate) mod shape_create_resource_snapshot_input;

pub(crate) mod shape_create_resource_snapshot_job_input;

pub(crate) mod shape_delete_resource_snapshot_job_input;

pub(crate) mod shape_disassociate_opportunity_input;

pub(crate) mod shape_get_aws_opportunity_summary_input;

pub(crate) mod shape_get_engagement_input;

pub(crate) mod shape_get_engagement_invitation_input;

pub(crate) mod shape_get_opportunity_input;

pub(crate) mod shape_get_resource_snapshot_input;

pub(crate) mod shape_get_resource_snapshot_job_input;

pub(crate) mod shape_get_selling_system_settings_input;

pub(crate) mod shape_internal_server_exception;

pub(crate) mod shape_list_engagement_by_accepting_invitation_tasks_input;

pub(crate) mod shape_list_engagement_from_opportunity_tasks_input;

pub(crate) mod shape_list_engagement_invitations_input;

pub(crate) mod shape_list_engagement_members_input;

pub(crate) mod shape_list_engagement_resource_associations_input;

pub(crate) mod shape_list_engagements_input;

pub(crate) mod shape_list_opportunities_input;

pub(crate) mod shape_list_resource_snapshot_jobs_input;

pub(crate) mod shape_list_resource_snapshots_input;

pub(crate) mod shape_list_solutions_input;

pub(crate) mod shape_list_tags_for_resource_input;

pub(crate) mod shape_put_selling_system_settings_input;

pub(crate) mod shape_reject_engagement_invitation_input;

pub(crate) mod shape_resource_not_found_exception;

pub(crate) mod shape_service_quota_exceeded_exception;

pub(crate) mod shape_start_engagement_by_accepting_invitation_task_input;

pub(crate) mod shape_start_engagement_from_opportunity_task_input;

pub(crate) mod shape_start_resource_snapshot_job_input;

pub(crate) mod shape_stop_resource_snapshot_job_input;

pub(crate) mod shape_submit_opportunity_input;

pub(crate) mod shape_tag_resource_input;

pub(crate) mod shape_throttling_exception;

pub(crate) mod shape_untag_resource_input;

pub(crate) mod shape_update_opportunity_input;

pub(crate) mod shape_validation_exception;

pub(crate) mod shape_assignee_contact;

pub(crate) mod shape_aws_opportunity_customer;

pub(crate) mod shape_aws_opportunity_insights;

pub(crate) mod shape_aws_opportunity_life_cycle;

pub(crate) mod shape_aws_opportunity_project;

pub(crate) mod shape_aws_opportunity_related_entities;

pub(crate) mod shape_aws_opportunity_team_members_list;

pub(crate) mod shape_aws_submission;

pub(crate) mod shape_contact;

pub(crate) mod shape_customer;

pub(crate) mod shape_engagement_context_details;

pub(crate) mod shape_engagement_contexts;

pub(crate) mod shape_engagement_invitation_summaries;

pub(crate) mod shape_engagement_member_summaries;

pub(crate) mod shape_engagement_members;

pub(crate) mod shape_engagement_resource_association_summary_list;

pub(crate) mod shape_engagement_sort;

pub(crate) mod shape_engagement_summary_list;

pub(crate) mod shape_invitation;

pub(crate) mod shape_last_modified_date;

pub(crate) mod shape_life_cycle;

pub(crate) mod shape_list_engagement_by_accepting_invitation_task_summaries;

pub(crate) mod shape_list_engagement_from_opportunity_task_summaries;

pub(crate) mod shape_list_tasks_sort_base;

pub(crate) mod shape_marketing;

pub(crate) mod shape_opportunity_engagement_invitation_sort;

pub(crate) mod shape_opportunity_sort;

pub(crate) mod shape_opportunity_summaries;

pub(crate) mod shape_partner_opportunity_team_members_list;

pub(crate) mod shape_payload;

pub(crate) mod shape_primary_needs_from_aws;

pub(crate) mod shape_project;

pub(crate) mod shape_receiver;

pub(crate) mod shape_related_entity_identifiers;

pub(crate) mod shape_resource_snapshot_job_summary_list;

pub(crate) mod shape_resource_snapshot_payload;

pub(crate) mod shape_resource_snapshot_summary_list;

pub(crate) mod shape_software_revenue;

pub(crate) mod shape_solution_list;

pub(crate) mod shape_solution_sort;

pub(crate) mod shape_sort_object;

pub(crate) mod shape_tag;

pub(crate) mod shape_tag_list;

pub(crate) mod shape_validation_exception_error_list;

pub(crate) mod shape_account;

pub(crate) mod shape_account_receiver;

pub(crate) mod shape_apn_programs;

pub(crate) mod shape_aws_marketplace_offer_identifiers;

pub(crate) mod shape_aws_product_identifiers;

pub(crate) mod shape_aws_team_member;

pub(crate) mod shape_channels;

pub(crate) mod shape_customer_contacts_list;

pub(crate) mod shape_delivery_models;

pub(crate) mod shape_engagement_context_payload;

pub(crate) mod shape_engagement_invitation_summary;

pub(crate) mod shape_engagement_member;

pub(crate) mod shape_engagement_member_summary;

pub(crate) mod shape_engagement_resource_association_summary;

pub(crate) mod shape_engagement_summary;

pub(crate) mod shape_expected_customer_spend;

pub(crate) mod shape_expected_customer_spend_list;

pub(crate) mod shape_list_engagement_by_accepting_invitation_task_summary;

pub(crate) mod shape_list_engagement_from_opportunity_task_summary;

pub(crate) mod shape_monetary_value;

pub(crate) mod shape_next_steps_histories;

pub(crate) mod shape_next_steps_history;

pub(crate) mod shape_opportunity_invitation_payload;

pub(crate) mod shape_opportunity_summary;

pub(crate) mod shape_opportunity_summary_view;

pub(crate) mod shape_profile_next_steps_histories;

pub(crate) mod shape_resource_snapshot_job_summary;

pub(crate) mod shape_resource_snapshot_summary;

pub(crate) mod shape_sales_activities;

pub(crate) mod shape_solution_base;

pub(crate) mod shape_solution_identifiers;

pub(crate) mod shape_use_cases;

pub(crate) mod shape_validation_exception_error;

pub(crate) mod shape_address;

pub(crate) mod shape_customer_projects_context;

pub(crate) mod shape_customer_summary;

pub(crate) mod shape_engagement_customer;

pub(crate) mod shape_life_cycle_for_view;

pub(crate) mod shape_life_cycle_summary;

pub(crate) mod shape_profile_next_steps_history;

pub(crate) mod shape_project_details;

pub(crate) mod shape_project_summary;

pub(crate) mod shape_project_view;

pub(crate) mod shape_receiver_responsibility_list;

pub(crate) mod shape_sender_contact_list;

pub(crate) mod shape_account_summary;

pub(crate) mod shape_engagement_customer_project_details;

pub(crate) mod shape_sender_contact;

pub(crate) mod shape_address_summary;

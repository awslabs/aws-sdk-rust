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

pub(crate) mod shape_accept_qualification_request;

pub(crate) mod shape_approve_assignment;

pub(crate) mod shape_associate_qualification_with_worker;

pub(crate) mod shape_create_additional_assignments_for_hit;

pub(crate) mod shape_create_hit;

pub(crate) mod shape_create_hit_type;

pub(crate) mod shape_create_hit_with_hit_type;

pub(crate) mod shape_create_qualification_type;

pub(crate) mod shape_create_worker_block;

pub(crate) mod shape_delete_hit;

pub(crate) mod shape_delete_qualification_type;

pub(crate) mod shape_delete_worker_block;

pub(crate) mod shape_disassociate_qualification_from_worker;

pub(crate) mod shape_get_account_balance;

pub(crate) mod shape_get_assignment;

pub(crate) mod shape_get_file_upload_url;

pub(crate) mod shape_get_hit;

pub(crate) mod shape_get_qualification_score;

pub(crate) mod shape_get_qualification_type;

pub(crate) mod shape_list_assignments_for_hit;

pub(crate) mod shape_list_bonus_payments;

pub(crate) mod shape_list_hits;

pub(crate) mod shape_list_hits_for_qualification_type;

pub(crate) mod shape_list_qualification_requests;

pub(crate) mod shape_list_qualification_types;

pub(crate) mod shape_list_review_policy_results_for_hit;

pub(crate) mod shape_list_reviewable_hits;

pub(crate) mod shape_list_worker_blocks;

pub(crate) mod shape_list_workers_with_qualification_type;

pub(crate) mod shape_notify_workers;

pub(crate) mod shape_reject_assignment;

pub(crate) mod shape_reject_qualification_request;

pub(crate) mod shape_send_bonus;

pub(crate) mod shape_send_test_event_notification;

pub(crate) mod shape_update_expiration_for_hit;

pub(crate) mod shape_update_hit_review_status;

pub(crate) mod shape_update_hit_type_of_hit;

pub(crate) mod shape_update_notification_settings;

pub(crate) mod shape_update_qualification_type;

pub(crate) mod shape_accept_qualification_request_input;

pub(crate) mod shape_approve_assignment_input;

pub(crate) mod shape_associate_qualification_with_worker_input;

pub(crate) mod shape_create_additional_assignments_for_hit_input;

pub(crate) fn or_empty_doc(data: &[u8]) -> &[u8] {
    if data.is_empty() {
        b"{}"
    } else {
        data
    }
}

pub(crate) mod shape_create_hit_input;

pub(crate) mod shape_create_hit_type_input;

pub(crate) mod shape_create_hit_with_hit_type_input;

pub(crate) mod shape_create_qualification_type_input;

pub(crate) mod shape_create_worker_block_input;

pub(crate) mod shape_delete_hit_input;

pub(crate) mod shape_delete_qualification_type_input;

pub(crate) mod shape_delete_worker_block_input;

pub(crate) mod shape_disassociate_qualification_from_worker_input;

pub(crate) mod shape_get_assignment_input;

pub(crate) mod shape_get_file_upload_url_input;

pub(crate) mod shape_get_hit_input;

pub(crate) mod shape_get_qualification_score_input;

pub(crate) mod shape_get_qualification_type_input;

pub(crate) mod shape_list_assignments_for_hit_input;

pub(crate) mod shape_list_bonus_payments_input;

pub(crate) mod shape_list_hits_for_qualification_type_input;

pub(crate) mod shape_list_hits_input;

pub(crate) mod shape_list_qualification_requests_input;

pub(crate) mod shape_list_qualification_types_input;

pub(crate) mod shape_list_review_policy_results_for_hit_input;

pub(crate) mod shape_list_reviewable_hits_input;

pub(crate) mod shape_list_worker_blocks_input;

pub(crate) mod shape_list_workers_with_qualification_type_input;

pub(crate) mod shape_notify_workers_input;

pub(crate) mod shape_reject_assignment_input;

pub(crate) mod shape_reject_qualification_request_input;

pub(crate) mod shape_request_error;

pub(crate) mod shape_send_bonus_input;

pub(crate) mod shape_send_test_event_notification_input;

pub(crate) mod shape_service_fault;

pub(crate) mod shape_update_expiration_for_hit_input;

pub(crate) mod shape_update_hit_review_status_input;

pub(crate) mod shape_update_hit_type_of_hit_input;

pub(crate) mod shape_update_notification_settings_input;

pub(crate) mod shape_update_qualification_type_input;

pub(crate) mod shape_assignment;

pub(crate) mod shape_assignment_list;

pub(crate) mod shape_bonus_payment_list;

pub(crate) mod shape_hit;

pub(crate) mod shape_hit_layout_parameter;

pub(crate) mod shape_hit_list;

pub(crate) mod shape_notification_specification;

pub(crate) mod shape_notify_workers_failure_status_list;

pub(crate) mod shape_qualification;

pub(crate) mod shape_qualification_list;

pub(crate) mod shape_qualification_request_list;

pub(crate) mod shape_qualification_requirement;

pub(crate) mod shape_qualification_type;

pub(crate) mod shape_qualification_type_list;

pub(crate) mod shape_review_policy;

pub(crate) mod shape_review_report;

pub(crate) mod shape_worker_block_list;

pub(crate) mod shape_bonus_payment;

pub(crate) mod shape_locale;

pub(crate) mod shape_notify_workers_failure_status;

pub(crate) mod shape_policy_parameter;

pub(crate) mod shape_policy_parameter_list;

pub(crate) mod shape_qualification_request;

pub(crate) mod shape_qualification_requirement_list;

pub(crate) mod shape_review_action_detail_list;

pub(crate) mod shape_review_result_detail_list;

pub(crate) mod shape_worker_block;

pub(crate) mod shape_parameter_map_entry;

pub(crate) mod shape_review_action_detail;

pub(crate) mod shape_review_result_detail;

pub(crate) mod shape_integer_list;

pub(crate) mod shape_locale_list;

pub(crate) mod shape_parameter_map_entry_list;

pub(crate) mod shape_string_list;

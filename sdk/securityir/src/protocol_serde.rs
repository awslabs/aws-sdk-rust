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

pub(crate) mod shape_batch_get_member_account_details;

pub(crate) mod shape_cancel_membership;

pub(crate) mod shape_close_case;

pub(crate) mod shape_create_case;

pub(crate) mod shape_create_case_comment;

pub(crate) mod shape_create_membership;

pub(crate) mod shape_get_case;

pub(crate) mod shape_get_case_attachment_download_url;

pub(crate) mod shape_get_case_attachment_upload_url;

pub(crate) mod shape_get_membership;

pub(crate) mod shape_list_case_edits;

pub(crate) mod shape_list_cases;

pub(crate) mod shape_list_comments;

pub(crate) mod shape_list_memberships;

pub(crate) mod shape_list_tags_for_resource;

pub(crate) mod shape_tag_resource;

pub(crate) mod shape_untag_resource;

pub(crate) mod shape_update_case;

pub(crate) mod shape_update_case_comment;

pub(crate) mod shape_update_case_status;

pub(crate) mod shape_update_membership;

pub(crate) mod shape_update_resolver_type;

pub(crate) fn or_empty_doc(data: &[u8]) -> &[u8] {
    if data.is_empty() {
        b"{}"
    } else {
        data
    }
}

pub(crate) mod shape_access_denied_exception;

pub(crate) mod shape_batch_get_member_account_details_input;

pub(crate) mod shape_conflict_exception;

pub(crate) mod shape_create_case_comment_input;

pub(crate) mod shape_create_case_input;

pub(crate) mod shape_create_membership_input;

pub(crate) mod shape_get_case_attachment_upload_url_input;

pub(crate) mod shape_internal_server_exception;

pub(crate) mod shape_invalid_token_exception;

pub(crate) mod shape_list_case_edits_input;

pub(crate) mod shape_list_cases_input;

pub(crate) mod shape_list_comments_input;

pub(crate) mod shape_list_memberships_input;

pub(crate) mod shape_resource_not_found_exception;

pub(crate) mod shape_security_incident_response_not_active_exception;

pub(crate) mod shape_service_quota_exceeded_exception;

pub(crate) mod shape_tag_resource_input;

pub(crate) mod shape_throttling_exception;

pub(crate) mod shape_update_case_comment_input;

pub(crate) mod shape_update_case_input;

pub(crate) mod shape_update_case_status_input;

pub(crate) mod shape_update_membership_input;

pub(crate) mod shape_update_resolver_type_input;

pub(crate) mod shape_validation_exception;

pub(crate) mod shape_case_attachments_list;

pub(crate) mod shape_case_edit_items;

pub(crate) mod shape_get_membership_account_detail_errors;

pub(crate) mod shape_get_membership_account_detail_items;

pub(crate) mod shape_impacted_accounts;

pub(crate) mod shape_impacted_aws_region;

pub(crate) mod shape_impacted_aws_region_list;

pub(crate) mod shape_impacted_services_list;

pub(crate) mod shape_incident_responder;

pub(crate) mod shape_incident_response_team;

pub(crate) mod shape_list_cases_items;

pub(crate) mod shape_list_comments_items;

pub(crate) mod shape_list_membership_items;

pub(crate) mod shape_opt_in_feature;

pub(crate) mod shape_opt_in_features;

pub(crate) mod shape_tag_map;

pub(crate) mod shape_threat_actor_ip;

pub(crate) mod shape_threat_actor_ip_list;

pub(crate) mod shape_validation_exception_field_list;

pub(crate) mod shape_watcher;

pub(crate) mod shape_watchers;

pub(crate) mod shape_case_attachment_attributes;

pub(crate) mod shape_case_edit_item;

pub(crate) mod shape_get_membership_account_detail_error;

pub(crate) mod shape_get_membership_account_detail_item;

pub(crate) mod shape_list_cases_item;

pub(crate) mod shape_list_comments_item;

pub(crate) mod shape_list_membership_item;

pub(crate) mod shape_validation_exception_field;

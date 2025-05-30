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

pub(crate) mod shape_associate_service_quota_template;

pub(crate) mod shape_create_support_case;

pub(crate) mod shape_delete_service_quota_increase_request_from_template;

pub(crate) mod shape_disassociate_service_quota_template;

pub(crate) mod shape_get_association_for_service_quota_template;

pub(crate) mod shape_get_aws_default_service_quota;

pub(crate) mod shape_get_requested_service_quota_change;

pub(crate) mod shape_get_service_quota;

pub(crate) mod shape_get_service_quota_increase_request_from_template;

pub(crate) mod shape_list_aws_default_service_quotas;

pub(crate) mod shape_list_requested_service_quota_change_history;

pub(crate) mod shape_list_requested_service_quota_change_history_by_quota;

pub(crate) mod shape_list_service_quota_increase_requests_in_template;

pub(crate) mod shape_list_service_quotas;

pub(crate) mod shape_list_services;

pub(crate) mod shape_list_tags_for_resource;

pub(crate) mod shape_put_service_quota_increase_request_into_template;

pub(crate) mod shape_request_service_quota_increase;

pub(crate) mod shape_tag_resource;

pub(crate) mod shape_untag_resource;

pub(crate) fn or_empty_doc(data: &[u8]) -> &[u8] {
    if data.is_empty() {
        b"{}"
    } else {
        data
    }
}

pub(crate) mod shape_access_denied_exception;

pub(crate) mod shape_aws_service_access_not_enabled_exception;

pub(crate) mod shape_create_support_case_input;

pub(crate) mod shape_delete_service_quota_increase_request_from_template_input;

pub(crate) mod shape_dependency_access_denied_exception;

pub(crate) mod shape_get_aws_default_service_quota_input;

pub(crate) mod shape_get_requested_service_quota_change_input;

pub(crate) mod shape_get_service_quota_increase_request_from_template_input;

pub(crate) mod shape_get_service_quota_input;

pub(crate) mod shape_illegal_argument_exception;

pub(crate) mod shape_invalid_pagination_token_exception;

pub(crate) mod shape_invalid_resource_state_exception;

pub(crate) mod shape_list_aws_default_service_quotas_input;

pub(crate) mod shape_list_requested_service_quota_change_history_by_quota_input;

pub(crate) mod shape_list_requested_service_quota_change_history_input;

pub(crate) mod shape_list_service_quota_increase_requests_in_template_input;

pub(crate) mod shape_list_service_quotas_input;

pub(crate) mod shape_list_services_input;

pub(crate) mod shape_list_tags_for_resource_input;

pub(crate) mod shape_no_available_organization_exception;

pub(crate) mod shape_no_such_resource_exception;

pub(crate) mod shape_organization_not_in_all_features_mode_exception;

pub(crate) mod shape_put_service_quota_increase_request_into_template_input;

pub(crate) mod shape_quota_exceeded_exception;

pub(crate) mod shape_request_service_quota_increase_input;

pub(crate) mod shape_resource_already_exists_exception;

pub(crate) mod shape_service_exception;

pub(crate) mod shape_service_quota_template_not_in_use_exception;

pub(crate) mod shape_tag_policy_violation_exception;

pub(crate) mod shape_tag_resource_input;

pub(crate) mod shape_templates_not_available_in_region_exception;

pub(crate) mod shape_too_many_requests_exception;

pub(crate) mod shape_too_many_tags_exception;

pub(crate) mod shape_untag_resource_input;

pub(crate) mod shape_output_tags;

pub(crate) mod shape_requested_service_quota_change;

pub(crate) mod shape_requested_service_quota_change_history_list_definition;

pub(crate) mod shape_service_info_list_definition;

pub(crate) mod shape_service_quota;

pub(crate) mod shape_service_quota_increase_request_in_template;

pub(crate) mod shape_service_quota_increase_request_in_template_list;

pub(crate) mod shape_service_quota_list_definition;

pub(crate) mod shape_tag;

pub(crate) mod shape_error_reason;

pub(crate) mod shape_metric_info;

pub(crate) mod shape_quota_context_info;

pub(crate) mod shape_quota_period;

pub(crate) mod shape_service_info;

pub(crate) mod shape_metric_dimensions_map_definition;

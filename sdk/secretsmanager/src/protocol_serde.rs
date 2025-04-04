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

pub(crate) mod shape_batch_get_secret_value;

pub(crate) mod shape_cancel_rotate_secret;

pub(crate) mod shape_create_secret;

pub(crate) mod shape_delete_resource_policy;

pub(crate) mod shape_delete_secret;

pub(crate) mod shape_describe_secret;

pub(crate) mod shape_get_random_password;

pub(crate) mod shape_get_resource_policy;

pub(crate) mod shape_get_secret_value;

pub(crate) mod shape_list_secret_version_ids;

pub(crate) mod shape_list_secrets;

pub(crate) mod shape_put_resource_policy;

pub(crate) mod shape_put_secret_value;

pub(crate) mod shape_remove_regions_from_replication;

pub(crate) mod shape_replicate_secret_to_regions;

pub(crate) mod shape_restore_secret;

pub(crate) mod shape_rotate_secret;

pub(crate) mod shape_stop_replication_to_replica;

pub(crate) mod shape_tag_resource;

pub(crate) mod shape_untag_resource;

pub(crate) mod shape_update_secret;

pub(crate) mod shape_update_secret_version_stage;

pub(crate) mod shape_validate_resource_policy;

pub(crate) fn or_empty_doc(data: &[u8]) -> &[u8] {
    if data.is_empty() {
        b"{}"
    } else {
        data
    }
}

pub(crate) mod shape_batch_get_secret_value_input;

pub(crate) mod shape_cancel_rotate_secret_input;

pub(crate) mod shape_create_secret_input;

pub(crate) mod shape_decryption_failure;

pub(crate) mod shape_delete_resource_policy_input;

pub(crate) mod shape_delete_secret_input;

pub(crate) mod shape_describe_secret_input;

pub(crate) mod shape_encryption_failure;

pub(crate) mod shape_get_random_password_input;

pub(crate) mod shape_get_resource_policy_input;

pub(crate) mod shape_get_secret_value_input;

pub(crate) mod shape_internal_service_error;

pub(crate) mod shape_invalid_next_token_exception;

pub(crate) mod shape_invalid_parameter_exception;

pub(crate) mod shape_invalid_request_exception;

pub(crate) mod shape_limit_exceeded_exception;

pub(crate) mod shape_list_secret_version_ids_input;

pub(crate) mod shape_list_secrets_input;

pub(crate) mod shape_malformed_policy_document_exception;

pub(crate) mod shape_precondition_not_met_exception;

pub(crate) mod shape_public_policy_exception;

pub(crate) mod shape_put_resource_policy_input;

pub(crate) mod shape_put_secret_value_input;

pub(crate) mod shape_remove_regions_from_replication_input;

pub(crate) mod shape_replicate_secret_to_regions_input;

pub(crate) mod shape_resource_exists_exception;

pub(crate) mod shape_resource_not_found_exception;

pub(crate) mod shape_restore_secret_input;

pub(crate) mod shape_rotate_secret_input;

pub(crate) mod shape_stop_replication_to_replica_input;

pub(crate) mod shape_tag_resource_input;

pub(crate) mod shape_untag_resource_input;

pub(crate) mod shape_update_secret_input;

pub(crate) mod shape_update_secret_version_stage_input;

pub(crate) mod shape_validate_resource_policy_input;

pub(crate) mod shape_api_error_list_type;

pub(crate) mod shape_filter;

pub(crate) mod shape_replica_region_type;

pub(crate) mod shape_replication_status_list_type;

pub(crate) mod shape_rotation_rules_type;

pub(crate) mod shape_secret_list_type;

pub(crate) mod shape_secret_values_type;

pub(crate) mod shape_secret_version_stages_type;

pub(crate) mod shape_secret_versions_list_type;

pub(crate) mod shape_secret_versions_to_stages_map_type;

pub(crate) mod shape_tag;

pub(crate) mod shape_tag_list_type;

pub(crate) mod shape_validation_errors_type;

pub(crate) mod shape_api_error_type;

pub(crate) mod shape_replication_status_type;

pub(crate) mod shape_secret_list_entry;

pub(crate) mod shape_secret_value_entry;

pub(crate) mod shape_secret_versions_list_entry;

pub(crate) mod shape_validation_errors_entry;

pub(crate) mod shape_kms_key_id_list_type;

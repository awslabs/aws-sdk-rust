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

pub(crate) mod shape_create_db_cluster;

pub(crate) mod shape_create_db_instance;

pub(crate) mod shape_create_db_parameter_group;

pub(crate) mod shape_delete_db_cluster;

pub(crate) mod shape_delete_db_instance;

pub(crate) mod shape_get_db_cluster;

pub(crate) mod shape_get_db_instance;

pub(crate) mod shape_get_db_parameter_group;

pub(crate) mod shape_list_db_clusters;

pub(crate) mod shape_list_db_instances;

pub(crate) mod shape_list_db_instances_for_cluster;

pub(crate) mod shape_list_db_parameter_groups;

pub(crate) mod shape_list_tags_for_resource;

pub(crate) mod shape_tag_resource;

pub(crate) mod shape_untag_resource;

pub(crate) mod shape_update_db_cluster;

pub(crate) mod shape_update_db_instance;

pub(crate) fn or_empty_doc(data: &[u8]) -> &[u8] {
    if data.is_empty() {
        b"{}"
    } else {
        data
    }
}

pub(crate) mod shape_access_denied_exception;

pub(crate) mod shape_conflict_exception;

pub(crate) mod shape_create_db_cluster_input;

pub(crate) mod shape_create_db_instance_input;

pub(crate) mod shape_create_db_parameter_group_input;

pub(crate) mod shape_delete_db_cluster_input;

pub(crate) mod shape_delete_db_instance_input;

pub(crate) mod shape_get_db_cluster_input;

pub(crate) mod shape_get_db_instance_input;

pub(crate) mod shape_get_db_parameter_group_input;

pub(crate) mod shape_internal_server_exception;

pub(crate) mod shape_list_db_clusters_input;

pub(crate) mod shape_list_db_instances_for_cluster_input;

pub(crate) mod shape_list_db_instances_input;

pub(crate) mod shape_list_db_parameter_groups_input;

pub(crate) mod shape_list_tags_for_resource_input;

pub(crate) mod shape_resource_not_found_exception;

pub(crate) mod shape_service_quota_exceeded_exception;

pub(crate) mod shape_tag_resource_input;

pub(crate) mod shape_throttling_exception;

pub(crate) mod shape_untag_resource_input;

pub(crate) mod shape_update_db_cluster_input;

pub(crate) mod shape_update_db_instance_input;

pub(crate) mod shape_validation_exception;

pub(crate) mod shape_db_cluster_summary_list;

pub(crate) mod shape_db_instance_for_cluster_summary_list;

pub(crate) mod shape_db_instance_summary_list;

pub(crate) mod shape_db_parameter_group_summary_list;

pub(crate) mod shape_log_delivery_configuration;

pub(crate) mod shape_parameters;

pub(crate) mod shape_response_tag_map;

pub(crate) mod shape_vpc_security_group_id_list;

pub(crate) mod shape_vpc_subnet_id_list;

pub(crate) mod shape_db_cluster_summary;

pub(crate) mod shape_db_instance_for_cluster_summary;

pub(crate) mod shape_db_instance_summary;

pub(crate) mod shape_db_parameter_group_summary;

pub(crate) mod shape_influx_dbv2_parameters;

pub(crate) mod shape_s3_configuration;

pub(crate) mod shape_duration;

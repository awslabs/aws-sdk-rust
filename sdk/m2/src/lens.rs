// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn reflens_structure_crate_output_list_applications_output_next_token(
    input: &crate::output::ListApplicationsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_application_versions_output_next_token(
    input: &crate::output::ListApplicationVersionsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_batch_job_definitions_output_next_token(
    input: &crate::output::ListBatchJobDefinitionsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_batch_job_executions_output_next_token(
    input: &crate::output::ListBatchJobExecutionsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_data_set_import_history_output_next_token(
    input: &crate::output::ListDataSetImportHistoryOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_data_sets_output_next_token(
    input: &crate::output::ListDataSetsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_deployments_output_next_token(
    input: &crate::output::ListDeploymentsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_engine_versions_output_next_token(
    input: &crate::output::ListEngineVersionsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_environments_output_next_token(
    input: &crate::output::ListEnvironmentsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_applications_output_applications(
    input: crate::output::ListApplicationsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::ApplicationSummary>> {
    let input = match input.applications {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_application_versions_output_application_versions(
    input: crate::output::ListApplicationVersionsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::ApplicationVersionSummary>> {
    let input = match input.application_versions {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_batch_job_definitions_output_batch_job_definitions(
    input: crate::output::ListBatchJobDefinitionsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::BatchJobDefinition>> {
    let input = match input.batch_job_definitions {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_batch_job_executions_output_batch_job_executions(
    input: crate::output::ListBatchJobExecutionsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::BatchJobExecutionSummary>> {
    let input = match input.batch_job_executions {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_data_set_import_history_output_data_set_import_tasks(
    input: crate::output::ListDataSetImportHistoryOutput,
) -> std::option::Option<std::vec::Vec<crate::model::DataSetImportTask>> {
    let input = match input.data_set_import_tasks {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_data_sets_output_data_sets(
    input: crate::output::ListDataSetsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::DataSetSummary>> {
    let input = match input.data_sets {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_deployments_output_deployments(
    input: crate::output::ListDeploymentsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::DeploymentSummary>> {
    let input = match input.deployments {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_engine_versions_output_engine_versions(
    input: crate::output::ListEngineVersionsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::EngineVersionsSummary>> {
    let input = match input.engine_versions {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_environments_output_environments(
    input: crate::output::ListEnvironmentsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::EnvironmentSummary>> {
    let input = match input.environments {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

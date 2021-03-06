// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn reflens_structure_crate_output_describe_batch_predictions_output_next_token(
    input: &crate::output::DescribeBatchPredictionsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_describe_data_sources_output_next_token(
    input: &crate::output::DescribeDataSourcesOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_describe_evaluations_output_next_token(
    input: &crate::output::DescribeEvaluationsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_describe_ml_models_output_next_token(
    input: &crate::output::DescribeMlModelsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_describe_batch_predictions_output_results(
    input: crate::output::DescribeBatchPredictionsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::BatchPrediction>> {
    let input = match input.results {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_describe_data_sources_output_results(
    input: crate::output::DescribeDataSourcesOutput,
) -> std::option::Option<std::vec::Vec<crate::model::DataSource>> {
    let input = match input.results {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_describe_evaluations_output_results(
    input: crate::output::DescribeEvaluationsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::Evaluation>> {
    let input = match input.results {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_describe_ml_models_output_results(
    input: crate::output::DescribeMlModelsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::MlModel>> {
    let input = match input.results {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

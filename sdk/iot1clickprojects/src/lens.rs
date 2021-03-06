// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn reflens_structure_crate_output_list_placements_output_next_token(
    input: &crate::output::ListPlacementsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_projects_output_next_token(
    input: &crate::output::ListProjectsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_placements_output_placements(
    input: crate::output::ListPlacementsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::PlacementSummary>> {
    let input = match input.placements {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_projects_output_projects(
    input: crate::output::ListProjectsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::ProjectSummary>> {
    let input = match input.projects {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

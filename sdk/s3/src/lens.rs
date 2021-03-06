// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn reflens_structure_crate_output_list_objects_v2_output_next_continuation_token(
    input: &crate::output::ListObjectsV2Output,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_continuation_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_parts_output_next_part_number_marker(
    input: &crate::output::ListPartsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_part_number_marker {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_parts_output_parts(
    input: crate::output::ListPartsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::Part>> {
    let input = match input.parts {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn reflens_structure_crate_output_list_dead_letter_source_queues_output_next_token(
    input: &crate::output::ListDeadLetterSourceQueuesOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_queues_output_next_token(
    input: &crate::output::ListQueuesOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_dead_letter_source_queues_output_queue_urls(
    input: crate::output::ListDeadLetterSourceQueuesOutput,
) -> std::option::Option<std::vec::Vec<std::string::String>> {
    let input = match input.queue_urls {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_queues_output_queue_urls(
    input: crate::output::ListQueuesOutput,
) -> std::option::Option<std::vec::Vec<std::string::String>> {
    let input = match input.queue_urls {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

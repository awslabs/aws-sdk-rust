// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn reflens_structure_crate_output_list_domains_output_next_page_marker(
    input: &crate::output::ListDomainsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_page_marker {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_operations_output_next_page_marker(
    input: &crate::output::ListOperationsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_page_marker {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_prices_output_next_page_marker(
    input: &crate::output::ListPricesOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_page_marker {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_view_billing_output_next_page_marker(
    input: &crate::output::ViewBillingOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_page_marker {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_domains_output_domains(
    input: crate::output::ListDomainsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::DomainSummary>> {
    let input = match input.domains {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_operations_output_operations(
    input: crate::output::ListOperationsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::OperationSummary>> {
    let input = match input.operations {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_prices_output_prices(
    input: crate::output::ListPricesOutput,
) -> std::option::Option<std::vec::Vec<crate::model::DomainPrice>> {
    let input = match input.prices {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_view_billing_output_billing_records(
    input: crate::output::ViewBillingOutput,
) -> std::option::Option<std::vec::Vec<crate::model::BillingRecord>> {
    let input = match input.billing_records {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

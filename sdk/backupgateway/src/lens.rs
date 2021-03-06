// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn reflens_structure_crate_output_list_gateways_output_next_token(
    input: &crate::output::ListGatewaysOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_hypervisors_output_next_token(
    input: &crate::output::ListHypervisorsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_list_virtual_machines_output_next_token(
    input: &crate::output::ListVirtualMachinesOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.next_token {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_gateways_output_gateways(
    input: crate::output::ListGatewaysOutput,
) -> std::option::Option<std::vec::Vec<crate::model::Gateway>> {
    let input = match input.gateways {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_hypervisors_output_hypervisors(
    input: crate::output::ListHypervisorsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::Hypervisor>> {
    let input = match input.hypervisors {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_list_virtual_machines_output_virtual_machines(
    input: crate::output::ListVirtualMachinesOutput,
) -> std::option::Option<std::vec::Vec<crate::model::VirtualMachine>> {
    let input = match input.virtual_machines {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

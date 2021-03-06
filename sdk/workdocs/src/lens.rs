// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn reflens_structure_crate_output_describe_document_versions_output_marker(
    input: &crate::output::DescribeDocumentVersionsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.marker {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_describe_folder_contents_output_marker(
    input: &crate::output::DescribeFolderContentsOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.marker {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn reflens_structure_crate_output_describe_users_output_marker(
    input: &crate::output::DescribeUsersOutput,
) -> std::option::Option<&std::string::String> {
    let input = match &input.marker {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_describe_document_versions_output_document_versions(
    input: crate::output::DescribeDocumentVersionsOutput,
) -> std::option::Option<std::vec::Vec<crate::model::DocumentVersionMetadata>> {
    let input = match input.document_versions {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

pub(crate) fn lens_structure_crate_output_describe_users_output_users(
    input: crate::output::DescribeUsersOutput,
) -> std::option::Option<std::vec::Vec<crate::model::User>> {
    let input = match input.users {
        None => return None,
        Some(t) => t,
    };
    Some(input)
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub(crate) fn reflens_list_certificates_output_output_next_token(
    input: &crate::operation::list_certificates::ListCertificatesOutput,
) -> ::std::option::Option<&::std::string::String> {
    let input = match &input.next_token {
        ::std::option::Option::None => return ::std::option::Option::None,
        ::std::option::Option::Some(t) => t,
    };
    ::std::option::Option::Some(input)
}

pub(crate) fn lens_list_certificates_output_output_certificate_summary_list(
    input: crate::operation::list_certificates::ListCertificatesOutput,
) -> ::std::option::Option<::std::vec::Vec<crate::types::CertificateSummary>> {
    let input = input.certificate_summary_list?;
    ::std::option::Option::Some(input)
}

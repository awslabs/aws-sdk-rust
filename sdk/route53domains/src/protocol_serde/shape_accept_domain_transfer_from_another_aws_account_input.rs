// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_accept_domain_transfer_from_another_aws_account_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::accept_domain_transfer_from_another_aws_account::AcceptDomainTransferFromAnotherAwsAccountInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.domain_name {
        object.key("DomainName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.password {
        object.key("Password").string(var_2.as_str());
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_start_address_list_import_job_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::start_address_list_import_job::StartAddressListImportJobInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.job_id {
        object.key("JobId").string(var_1.as_str());
    }
    Ok(())
}

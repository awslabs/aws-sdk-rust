// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_delete_report_group_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::delete_report_group::DeleteReportGroupInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.arn {
        object.key("arn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.delete_reports {
        object.key("deleteReports").boolean(*var_2);
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_describe_edge_packaging_job_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::describe_edge_packaging_job::DescribeEdgePackagingJobInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.edge_packaging_job_name {
        object.key("EdgePackagingJobName").string(var_1.as_str());
    }
    Ok(())
}

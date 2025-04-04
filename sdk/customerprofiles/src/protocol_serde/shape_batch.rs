// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_batch(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::Batch,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object
            .key("StartTime")
            .date_time(&input.start_time, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    {
        object
            .key("EndTime")
            .date_time(&input.end_time, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    Ok(())
}

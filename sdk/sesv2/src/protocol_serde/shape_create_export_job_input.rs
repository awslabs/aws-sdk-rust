// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_export_job_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_export_job::CreateExportJobInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.export_data_source {
        #[allow(unused_mut)]
        let mut object_2 = object.key("ExportDataSource").start_object();
        crate::protocol_serde::shape_export_data_source::ser_export_data_source(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.export_destination {
        #[allow(unused_mut)]
        let mut object_4 = object.key("ExportDestination").start_object();
        crate::protocol_serde::shape_export_destination::ser_export_destination(&mut object_4, var_3)?;
        object_4.finish();
    }
    Ok(())
}

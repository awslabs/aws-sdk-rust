// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_import_job_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_import_job::CreateImportJobInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.import_data_source {
        #[allow(unused_mut)]
        let mut object_2 = object.key("ImportDataSource").start_object();
        crate::protocol_serde::shape_import_data_source::ser_import_data_source(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.import_destination {
        #[allow(unused_mut)]
        let mut object_4 = object.key("ImportDestination").start_object();
        crate::protocol_serde::shape_import_destination::ser_import_destination(&mut object_4, var_3)?;
        object_4.finish();
    }
    Ok(())
}

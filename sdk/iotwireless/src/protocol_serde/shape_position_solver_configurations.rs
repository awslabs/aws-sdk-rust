// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_position_solver_configurations(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::PositionSolverConfigurations,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.semtech_gnss {
        #[allow(unused_mut)]
        let mut object_2 = object.key("SemtechGnss").start_object();
        crate::protocol_serde::shape_semtech_gnss_configuration::ser_semtech_gnss_configuration(&mut object_2, var_1)?;
        object_2.finish();
    }
    Ok(())
}

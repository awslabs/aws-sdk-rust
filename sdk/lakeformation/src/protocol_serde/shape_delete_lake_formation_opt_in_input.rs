// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_delete_lake_formation_opt_in_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::delete_lake_formation_opt_in::DeleteLakeFormationOptInInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.condition {
        #[allow(unused_mut)]
        let mut object_2 = object.key("Condition").start_object();
        crate::protocol_serde::shape_condition::ser_condition(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.principal {
        #[allow(unused_mut)]
        let mut object_4 = object.key("Principal").start_object();
        crate::protocol_serde::shape_data_lake_principal::ser_data_lake_principal(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.resource {
        #[allow(unused_mut)]
        let mut object_6 = object.key("Resource").start_object();
        crate::protocol_serde::shape_resource::ser_resource(&mut object_6, var_5)?;
        object_6.finish();
    }
    Ok(())
}

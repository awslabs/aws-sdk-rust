// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_f_ports(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::UpdateFPorts,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.positioning {
        #[allow(unused_mut)]
        let mut object_2 = object.key("Positioning").start_object();
        crate::protocol_serde::shape_positioning::ser_positioning(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.applications {
        let mut array_4 = object.key("Applications").start_array();
        for item_5 in var_3 {
            {
                #[allow(unused_mut)]
                let mut object_6 = array_4.value().start_object();
                crate::protocol_serde::shape_application_config::ser_application_config(&mut object_6, item_5)?;
                object_6.finish();
            }
        }
        array_4.finish();
    }
    Ok(())
}

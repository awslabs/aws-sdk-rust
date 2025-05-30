// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_integration_resource_property_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_integration_resource_property::CreateIntegrationResourcePropertyInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.resource_arn {
        object.key("ResourceArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.source_processing_properties {
        #[allow(unused_mut)]
        let mut object_3 = object.key("SourceProcessingProperties").start_object();
        crate::protocol_serde::shape_source_processing_properties::ser_source_processing_properties(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.target_processing_properties {
        #[allow(unused_mut)]
        let mut object_5 = object.key("TargetProcessingProperties").start_object();
        crate::protocol_serde::shape_target_processing_properties::ser_target_processing_properties(&mut object_5, var_4)?;
        object_5.finish();
    }
    Ok(())
}

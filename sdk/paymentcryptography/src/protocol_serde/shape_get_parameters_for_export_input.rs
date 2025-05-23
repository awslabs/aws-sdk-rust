// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_get_parameters_for_export_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::get_parameters_for_export::GetParametersForExportInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.key_material_type {
        object.key("KeyMaterialType").string(var_1.as_str());
    }
    if let Some(var_2) = &input.signing_key_algorithm {
        object.key("SigningKeyAlgorithm").string(var_2.as_str());
    }
    Ok(())
}

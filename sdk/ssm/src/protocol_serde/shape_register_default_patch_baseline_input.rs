// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_register_default_patch_baseline_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::register_default_patch_baseline::RegisterDefaultPatchBaselineInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.baseline_id {
        object.key("BaselineId").string(var_1.as_str());
    }
    Ok(())
}

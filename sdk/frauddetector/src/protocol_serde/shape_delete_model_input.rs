// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_delete_model_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::delete_model::DeleteModelInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.model_id {
        object.key("modelId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.model_type {
        object.key("modelType").string(var_2.as_str());
    }
    Ok(())
}

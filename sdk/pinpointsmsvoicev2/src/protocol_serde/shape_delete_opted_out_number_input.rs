// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_delete_opted_out_number_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::delete_opted_out_number::DeleteOptedOutNumberInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.opt_out_list_name {
        object.key("OptOutListName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.opted_out_number {
        object.key("OptedOutNumber").string(var_2.as_str());
    }
    Ok(())
}

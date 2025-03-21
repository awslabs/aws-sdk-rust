// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_describe_principal_mapping_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::describe_principal_mapping::DescribePrincipalMappingInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.index_id {
        object.key("IndexId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.data_source_id {
        object.key("DataSourceId").string(var_2.as_str());
    }
    if let Some(var_3) = &input.group_id {
        object.key("GroupId").string(var_3.as_str());
    }
    Ok(())
}

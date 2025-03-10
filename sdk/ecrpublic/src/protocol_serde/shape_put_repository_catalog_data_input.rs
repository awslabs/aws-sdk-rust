// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_put_repository_catalog_data_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::put_repository_catalog_data::PutRepositoryCatalogDataInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.registry_id {
        object.key("registryId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.repository_name {
        object.key("repositoryName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.catalog_data {
        #[allow(unused_mut)]
        let mut object_4 = object.key("catalogData").start_object();
        crate::protocol_serde::shape_repository_catalog_data_input::ser_repository_catalog_data_input(&mut object_4, var_3)?;
        object_4.finish();
    }
    Ok(())
}

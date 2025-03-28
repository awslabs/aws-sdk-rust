// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_associate_source_graphql_api_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::associate_source_graphql_api::AssociateSourceGraphqlApiInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.description {
        object.key("description").string(var_1.as_str());
    }
    if let Some(var_2) = &input.source_api_association_config {
        #[allow(unused_mut)]
        let mut object_3 = object.key("sourceApiAssociationConfig").start_object();
        crate::protocol_serde::shape_source_api_association_config::ser_source_api_association_config(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.source_api_identifier {
        object.key("sourceApiIdentifier").string(var_4.as_str());
    }
    Ok(())
}

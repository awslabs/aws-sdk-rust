// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_retrieve_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::retrieve::RetrieveInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.guardrail_configuration {
        #[allow(unused_mut)]
        let mut object_2 = object.key("guardrailConfiguration").start_object();
        crate::protocol_serde::shape_guardrail_configuration::ser_guardrail_configuration(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.next_token {
        object.key("nextToken").string(var_3.as_str());
    }
    if let Some(var_4) = &input.retrieval_configuration {
        #[allow(unused_mut)]
        let mut object_5 = object.key("retrievalConfiguration").start_object();
        crate::protocol_serde::shape_knowledge_base_retrieval_configuration::ser_knowledge_base_retrieval_configuration(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.retrieval_query {
        #[allow(unused_mut)]
        let mut object_7 = object.key("retrievalQuery").start_object();
        crate::protocol_serde::shape_knowledge_base_query::ser_knowledge_base_query(&mut object_7, var_6)?;
        object_7.finish();
    }
    Ok(())
}

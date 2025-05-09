// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_glossary_term_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_glossary_term::CreateGlossaryTermInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.client_token {
        object.key("clientToken").string(var_1.as_str());
    }
    if let Some(var_2) = &input.glossary_identifier {
        object.key("glossaryIdentifier").string(var_2.as_str());
    }
    if let Some(var_3) = &input.long_description {
        object.key("longDescription").string(var_3.as_str());
    }
    if let Some(var_4) = &input.name {
        object.key("name").string(var_4.as_str());
    }
    if let Some(var_5) = &input.short_description {
        object.key("shortDescription").string(var_5.as_str());
    }
    if let Some(var_6) = &input.status {
        object.key("status").string(var_6.as_str());
    }
    if let Some(var_7) = &input.term_relations {
        #[allow(unused_mut)]
        let mut object_8 = object.key("termRelations").start_object();
        crate::protocol_serde::shape_term_relations::ser_term_relations(&mut object_8, var_7)?;
        object_8.finish();
    }
    Ok(())
}

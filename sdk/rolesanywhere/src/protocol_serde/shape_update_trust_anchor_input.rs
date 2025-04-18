// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_trust_anchor_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_trust_anchor::UpdateTrustAnchorInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.name {
        object.key("name").string(var_1.as_str());
    }
    if let Some(var_2) = &input.source {
        #[allow(unused_mut)]
        let mut object_3 = object.key("source").start_object();
        crate::protocol_serde::shape_source::ser_source(&mut object_3, var_2)?;
        object_3.finish();
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_disassociate_trial_component_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::disassociate_trial_component::DisassociateTrialComponentInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.trial_component_name {
        object.key("TrialComponentName").string(var_1.as_str());
    }
    if let Some(var_2) = &input.trial_name {
        object.key("TrialName").string(var_2.as_str());
    }
    Ok(())
}

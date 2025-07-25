// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_space_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_space::UpdateSpaceInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.description {
        object.key("description").string(var_1.as_str());
    }
    if let Some(var_2) = &input.role_arn {
        object.key("roleArn").string(var_2.as_str());
    }
    if let Some(var_3) = &input.supported_email_domains {
        #[allow(unused_mut)]
        let mut object_4 = object.key("supportedEmailDomains").start_object();
        crate::protocol_serde::shape_supported_email_domains_parameters::ser_supported_email_domains_parameters(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.tier {
        object.key("tier").string(var_5.as_str());
    }
    Ok(())
}

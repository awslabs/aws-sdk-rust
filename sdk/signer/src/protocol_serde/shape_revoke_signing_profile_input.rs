// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_revoke_signing_profile_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::revoke_signing_profile::RevokeSigningProfileInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.effective_time {
        object
            .key("effectiveTime")
            .date_time(var_1, ::aws_smithy_types::date_time::Format::EpochSeconds)?;
    }
    if let Some(var_2) = &input.profile_version {
        object.key("profileVersion").string(var_2.as_str());
    }
    if let Some(var_3) = &input.reason {
        object.key("reason").string(var_3.as_str());
    }
    Ok(())
}

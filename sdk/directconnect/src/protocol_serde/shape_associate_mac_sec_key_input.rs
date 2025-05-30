// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_associate_mac_sec_key_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::associate_mac_sec_key::AssociateMacSecKeyInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.connection_id {
        object.key("connectionId").string(var_1.as_str());
    }
    if let Some(var_2) = &input.secret_arn {
        object.key("secretARN").string(var_2.as_str());
    }
    if let Some(var_3) = &input.ckn {
        object.key("ckn").string(var_3.as_str());
    }
    if let Some(var_4) = &input.cak {
        object.key("cak").string(var_4.as_str());
    }
    Ok(())
}

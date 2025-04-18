// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_ephemeris_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_ephemeris::UpdateEphemerisInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.enabled {
        object.key("enabled").boolean(*var_1);
    }
    if let Some(var_2) = &input.name {
        object.key("name").string(var_2.as_str());
    }
    if let Some(var_3) = &input.priority {
        object.key("priority").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_3).into()),
        );
    }
    Ok(())
}

// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_get_regex_pattern_set_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::get_regex_pattern_set::GetRegexPatternSetInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.regex_pattern_set_id {
        object.key("RegexPatternSetId").string(var_1.as_str());
    }
    Ok(())
}

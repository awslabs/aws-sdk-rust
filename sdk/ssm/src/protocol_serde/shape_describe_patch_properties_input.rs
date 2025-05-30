// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_describe_patch_properties_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::describe_patch_properties::DescribePatchPropertiesInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.operating_system {
        object.key("OperatingSystem").string(var_1.as_str());
    }
    if let Some(var_2) = &input.property {
        object.key("Property").string(var_2.as_str());
    }
    if let Some(var_3) = &input.patch_set {
        object.key("PatchSet").string(var_3.as_str());
    }
    if let Some(var_4) = &input.max_results {
        object.key("MaxResults").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_4).into()),
        );
    }
    if let Some(var_5) = &input.next_token {
        object.key("NextToken").string(var_5.as_str());
    }
    Ok(())
}

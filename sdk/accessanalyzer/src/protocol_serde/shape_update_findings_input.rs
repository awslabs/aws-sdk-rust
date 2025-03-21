// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_update_findings_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::update_findings::UpdateFindingsInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.analyzer_arn {
        object.key("analyzerArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.client_token {
        object.key("clientToken").string(var_2.as_str());
    }
    if let Some(var_3) = &input.ids {
        let mut array_4 = object.key("ids").start_array();
        for item_5 in var_3 {
            {
                array_4.value().string(item_5.as_str());
            }
        }
        array_4.finish();
    }
    if let Some(var_6) = &input.resource_arn {
        object.key("resourceArn").string(var_6.as_str());
    }
    if let Some(var_7) = &input.status {
        object.key("status").string(var_7.as_str());
    }
    Ok(())
}

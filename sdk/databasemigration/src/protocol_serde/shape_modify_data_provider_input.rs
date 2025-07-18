// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_modify_data_provider_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::modify_data_provider::ModifyDataProviderInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.data_provider_identifier {
        object.key("DataProviderIdentifier").string(var_1.as_str());
    }
    if let Some(var_2) = &input.data_provider_name {
        object.key("DataProviderName").string(var_2.as_str());
    }
    if let Some(var_3) = &input.description {
        object.key("Description").string(var_3.as_str());
    }
    if let Some(var_4) = &input.engine {
        object.key("Engine").string(var_4.as_str());
    }
    if let Some(var_5) = &input.r#virtual {
        object.key("Virtual").boolean(*var_5);
    }
    if let Some(var_6) = &input.exact_settings {
        object.key("ExactSettings").boolean(*var_6);
    }
    if let Some(var_7) = &input.settings {
        #[allow(unused_mut)]
        let mut object_8 = object.key("Settings").start_object();
        crate::protocol_serde::shape_data_provider_settings::ser_data_provider_settings(&mut object_8, var_7)?;
        object_8.finish();
    }
    Ok(())
}

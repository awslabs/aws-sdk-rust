// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_customer_profiles_integration_config(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::CustomerProfilesIntegrationConfig,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("domainArn").string(input.domain_arn.as_str());
    }
    {
        #[allow(unused_mut)]
        let mut object_1 = object.key("objectTypeNames").start_object();
        for (key_2, value_3) in &input.object_type_names {
            {
                object_1.key(key_2.as_str()).string(value_3.as_str());
            }
        }
        object_1.finish();
    }
    Ok(())
}

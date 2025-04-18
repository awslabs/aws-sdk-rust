// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_organization_data_source_configurations(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::OrganizationDataSourceConfigurations,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.s3_logs {
        #[allow(unused_mut)]
        let mut object_2 = object.key("s3Logs").start_object();
        crate::protocol_serde::shape_organization_s3_logs_configuration::ser_organization_s3_logs_configuration(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.kubernetes {
        #[allow(unused_mut)]
        let mut object_4 = object.key("kubernetes").start_object();
        crate::protocol_serde::shape_organization_kubernetes_configuration::ser_organization_kubernetes_configuration(&mut object_4, var_3)?;
        object_4.finish();
    }
    if let Some(var_5) = &input.malware_protection {
        #[allow(unused_mut)]
        let mut object_6 = object.key("malwareProtection").start_object();
        crate::protocol_serde::shape_organization_malware_protection_configuration::ser_organization_malware_protection_configuration(
            &mut object_6,
            var_5,
        )?;
        object_6.finish();
    }
    Ok(())
}

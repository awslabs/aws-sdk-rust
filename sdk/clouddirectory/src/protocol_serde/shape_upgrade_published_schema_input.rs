// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_upgrade_published_schema_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::upgrade_published_schema::UpgradePublishedSchemaInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.development_schema_arn {
        object.key("DevelopmentSchemaArn").string(var_1.as_str());
    }
    if let Some(var_2) = &input.dry_run {
        object.key("DryRun").boolean(*var_2);
    }
    if let Some(var_3) = &input.minor_version {
        object.key("MinorVersion").string(var_3.as_str());
    }
    if let Some(var_4) = &input.published_schema_arn {
        object.key("PublishedSchemaArn").string(var_4.as_str());
    }
    Ok(())
}

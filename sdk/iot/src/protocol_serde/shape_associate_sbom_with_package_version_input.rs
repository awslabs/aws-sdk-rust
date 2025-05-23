// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_associate_sbom_with_package_version_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::associate_sbom_with_package_version::AssociateSbomWithPackageVersionInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.sbom {
        #[allow(unused_mut)]
        let mut object_2 = object.key("sbom").start_object();
        crate::protocol_serde::shape_sbom::ser_sbom(&mut object_2, var_1)?;
        object_2.finish();
    }
    Ok(())
}

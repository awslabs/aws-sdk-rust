// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_deregister_container_instance_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::deregister_container_instance::DeregisterContainerInstanceInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.cluster {
        object.key("cluster").string(var_1.as_str());
    }
    if let Some(var_2) = &input.container_instance {
        object.key("containerInstance").string(var_2.as_str());
    }
    if let Some(var_3) = &input.force {
        object.key("force").boolean(*var_3);
    }
    Ok(())
}

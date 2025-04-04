// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_sync_resource_filter(
    object_4: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::SyncResourceFilter,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    match input {
        crate::types::SyncResourceFilter::State(inner) => {
            object_4.key("state").string(inner.as_str());
        }
        crate::types::SyncResourceFilter::ResourceType(inner) => {
            object_4.key("resourceType").string(inner.as_str());
        }
        crate::types::SyncResourceFilter::ResourceId(inner) => {
            object_4.key("resourceId").string(inner.as_str());
        }
        crate::types::SyncResourceFilter::ExternalId(inner) => {
            object_4.key("externalId").string(inner.as_str());
        }
        crate::types::SyncResourceFilter::Unknown => {
            return Err(::aws_smithy_types::error::operation::SerializationError::unknown_variant(
                "SyncResourceFilter",
            ))
        }
    }
    Ok(())
}

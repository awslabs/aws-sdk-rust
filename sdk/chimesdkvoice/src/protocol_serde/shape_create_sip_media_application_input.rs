// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_create_sip_media_application_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::create_sip_media_application::CreateSipMediaApplicationInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.aws_region {
        object.key("AwsRegion").string(var_1.as_str());
    }
    if let Some(var_2) = &input.endpoints {
        let mut array_3 = object.key("Endpoints").start_array();
        for item_4 in var_2 {
            {
                #[allow(unused_mut)]
                let mut object_5 = array_3.value().start_object();
                crate::protocol_serde::shape_sip_media_application_endpoint::ser_sip_media_application_endpoint(&mut object_5, item_4)?;
                object_5.finish();
            }
        }
        array_3.finish();
    }
    if let Some(var_6) = &input.name {
        object.key("Name").string(var_6.as_str());
    }
    if let Some(var_7) = &input.tags {
        let mut array_8 = object.key("Tags").start_array();
        for item_9 in var_7 {
            {
                #[allow(unused_mut)]
                let mut object_10 = array_8.value().start_object();
                crate::protocol_serde::shape_tag::ser_tag(&mut object_10, item_9)?;
                object_10.finish();
            }
        }
        array_8.finish();
    }
    Ok(())
}

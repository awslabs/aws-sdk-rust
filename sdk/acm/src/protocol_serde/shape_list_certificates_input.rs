// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_list_certificates_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::list_certificates::ListCertificatesInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.certificate_statuses {
        let mut array_2 = object.key("CertificateStatuses").start_array();
        for item_3 in var_1 {
            {
                array_2.value().string(item_3.as_str());
            }
        }
        array_2.finish();
    }
    if let Some(var_4) = &input.includes {
        #[allow(unused_mut)]
        let mut object_5 = object.key("Includes").start_object();
        crate::protocol_serde::shape_filters::ser_filters(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.next_token {
        object.key("NextToken").string(var_6.as_str());
    }
    if let Some(var_7) = &input.max_items {
        object.key("MaxItems").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((*var_7).into()),
        );
    }
    if let Some(var_8) = &input.sort_by {
        object.key("SortBy").string(var_8.as_str());
    }
    if let Some(var_9) = &input.sort_order {
        object.key("SortOrder").string(var_9.as_str());
    }
    Ok(())
}

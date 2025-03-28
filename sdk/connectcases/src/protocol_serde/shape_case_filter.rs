// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_case_filter(
    object_6: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::CaseFilter,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    match input {
        crate::types::CaseFilter::Field(inner) => {
            #[allow(unused_mut)]
            let mut object_1 = object_6.key("field").start_object();
            crate::protocol_serde::shape_field_filter::ser_field_filter(&mut object_1, inner)?;
            object_1.finish();
        }
        crate::types::CaseFilter::Not(inner) => {
            #[allow(unused_mut)]
            let mut object_2 = object_6.key("not").start_object();
            crate::protocol_serde::shape_case_filter::ser_case_filter(&mut object_2, inner)?;
            object_2.finish();
        }
        crate::types::CaseFilter::AndAll(inner) => {
            let mut array_3 = object_6.key("andAll").start_array();
            for item_4 in inner {
                {
                    #[allow(unused_mut)]
                    let mut object_5 = array_3.value().start_object();
                    crate::protocol_serde::shape_case_filter::ser_case_filter(&mut object_5, item_4)?;
                    object_5.finish();
                }
            }
            array_3.finish();
        }
        crate::types::CaseFilter::OrAll(inner) => {
            let mut array_6 = object_6.key("orAll").start_array();
            for item_7 in inner {
                {
                    #[allow(unused_mut)]
                    let mut object_8 = array_6.value().start_object();
                    crate::protocol_serde::shape_case_filter::ser_case_filter(&mut object_8, item_7)?;
                    object_8.finish();
                }
            }
            array_6.finish();
        }
        crate::types::CaseFilter::Unknown => return Err(::aws_smithy_types::error::operation::SerializationError::unknown_variant("CaseFilter")),
    }
    Ok(())
}

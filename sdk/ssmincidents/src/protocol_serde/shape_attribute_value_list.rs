// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_attribute_value_list(
    object_1: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::AttributeValueList,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    match input {
        crate::types::AttributeValueList::StringValues(inner) => {
            let mut array_1 = object_1.key("stringValues").start_array();
            for item_2 in inner {
                {
                    array_1.value().string(item_2.as_str());
                }
            }
            array_1.finish();
        }
        crate::types::AttributeValueList::IntegerValues(inner) => {
            let mut array_3 = object_1.key("integerValues").start_array();
            for item_4 in inner {
                {
                    array_3.value().number(
                        #[allow(clippy::useless_conversion)]
                        ::aws_smithy_types::Number::NegInt((*item_4).into()),
                    );
                }
            }
            array_3.finish();
        }
        crate::types::AttributeValueList::Unknown => {
            return Err(::aws_smithy_types::error::operation::SerializationError::unknown_variant(
                "AttributeValueList",
            ))
        }
    }
    Ok(())
}

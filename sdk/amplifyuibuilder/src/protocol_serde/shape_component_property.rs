// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_component_property(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::ComponentProperty,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.value {
        object.key("value").string(var_1.as_str());
    }
    if let Some(var_2) = &input.binding_properties {
        #[allow(unused_mut)]
        let mut object_3 = object.key("bindingProperties").start_object();
        crate::protocol_serde::shape_component_property_binding_properties::ser_component_property_binding_properties(&mut object_3, var_2)?;
        object_3.finish();
    }
    if let Some(var_4) = &input.collection_binding_properties {
        #[allow(unused_mut)]
        let mut object_5 = object.key("collectionBindingProperties").start_object();
        crate::protocol_serde::shape_component_property_binding_properties::ser_component_property_binding_properties(&mut object_5, var_4)?;
        object_5.finish();
    }
    if let Some(var_6) = &input.default_value {
        object.key("defaultValue").string(var_6.as_str());
    }
    if let Some(var_7) = &input.model {
        object.key("model").string(var_7.as_str());
    }
    if let Some(var_8) = &input.bindings {
        #[allow(unused_mut)]
        let mut object_9 = object.key("bindings").start_object();
        for (key_10, value_11) in var_8 {
            {
                #[allow(unused_mut)]
                let mut object_12 = object_9.key(key_10.as_str()).start_object();
                crate::protocol_serde::shape_form_binding_element::ser_form_binding_element(&mut object_12, value_11)?;
                object_12.finish();
            }
        }
        object_9.finish();
    }
    if let Some(var_13) = &input.event {
        object.key("event").string(var_13.as_str());
    }
    if let Some(var_14) = &input.user_attribute {
        object.key("userAttribute").string(var_14.as_str());
    }
    if let Some(var_15) = &input.concat {
        let mut array_16 = object.key("concat").start_array();
        for item_17 in var_15 {
            {
                #[allow(unused_mut)]
                let mut object_18 = array_16.value().start_object();
                crate::protocol_serde::shape_component_property::ser_component_property(&mut object_18, item_17)?;
                object_18.finish();
            }
        }
        array_16.finish();
    }
    if let Some(var_19) = &input.condition {
        #[allow(unused_mut)]
        let mut object_20 = object.key("condition").start_object();
        crate::protocol_serde::shape_component_condition_property::ser_component_condition_property(&mut object_20, var_19)?;
        object_20.finish();
    }
    if let Some(var_21) = &input.configured {
        object.key("configured").boolean(*var_21);
    }
    if let Some(var_22) = &input.r#type {
        object.key("type").string(var_22.as_str());
    }
    if let Some(var_23) = &input.imported_value {
        object.key("importedValue").string(var_23.as_str());
    }
    if let Some(var_24) = &input.component_name {
        object.key("componentName").string(var_24.as_str());
    }
    if let Some(var_25) = &input.property {
        object.key("property").string(var_25.as_str());
    }
    Ok(())
}

pub(crate) fn de_component_property<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::ComponentProperty>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::ComponentPropertyBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "value" => {
                            builder = builder.set_value(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "bindingProperties" => {
                            builder = builder.set_binding_properties(
                                crate::protocol_serde::shape_component_property_binding_properties::de_component_property_binding_properties(tokens)?,
                            );
                        }
                        "collectionBindingProperties" => {
                            builder = builder.set_collection_binding_properties(
                                crate::protocol_serde::shape_component_property_binding_properties::de_component_property_binding_properties(tokens)?,
                            );
                        }
                        "defaultValue" => {
                            builder = builder.set_default_value(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "model" => {
                            builder = builder.set_model(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "bindings" => {
                            builder = builder.set_bindings(crate::protocol_serde::shape_form_bindings::de_form_bindings(tokens)?);
                        }
                        "event" => {
                            builder = builder.set_event(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "userAttribute" => {
                            builder = builder.set_user_attribute(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "concat" => {
                            builder = builder.set_concat(crate::protocol_serde::shape_component_property_list::de_component_property_list(tokens)?);
                        }
                        "condition" => {
                            builder = builder
                                .set_condition(crate::protocol_serde::shape_component_condition_property::de_component_condition_property(tokens)?);
                        }
                        "configured" => {
                            builder = builder.set_configured(::aws_smithy_json::deserialize::token::expect_bool_or_null(tokens.next())?);
                        }
                        "type" => {
                            builder = builder.set_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "importedValue" => {
                            builder = builder.set_imported_value(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "componentName" => {
                            builder = builder.set_component_name(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "property" => {
                            builder = builder.set_property(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        _ => ::aws_smithy_json::deserialize::token::skip_value(tokens)?,
                    },
                    other => {
                        return Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(format!(
                            "expected object key or end object, found: {:?}",
                            other
                        )))
                    }
                }
            }
            Ok(Some(builder.build()))
        }
        _ => Err(::aws_smithy_json::deserialize::error::DeserializeError::custom(
            "expected start object or null",
        )),
    }
}

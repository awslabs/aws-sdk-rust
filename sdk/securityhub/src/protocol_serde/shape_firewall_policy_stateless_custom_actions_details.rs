// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_firewall_policy_stateless_custom_actions_details(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::FirewallPolicyStatelessCustomActionsDetails,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.action_definition {
        #[allow(unused_mut)]
        let mut object_2 = object.key("ActionDefinition").start_object();
        crate::protocol_serde::shape_stateless_custom_action_definition::ser_stateless_custom_action_definition(&mut object_2, var_1)?;
        object_2.finish();
    }
    if let Some(var_3) = &input.action_name {
        object.key("ActionName").string(var_3.as_str());
    }
    Ok(())
}

pub(crate) fn de_firewall_policy_stateless_custom_actions_details<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::FirewallPolicyStatelessCustomActionsDetails>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::FirewallPolicyStatelessCustomActionsDetailsBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "ActionDefinition" => {
                            builder = builder.set_action_definition(
                                crate::protocol_serde::shape_stateless_custom_action_definition::de_stateless_custom_action_definition(tokens)?,
                            );
                        }
                        "ActionName" => {
                            builder = builder.set_action_name(
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

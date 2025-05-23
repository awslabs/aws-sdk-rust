// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_indonesia_additional_info(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::IndonesiaAdditionalInfo,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.tax_registration_number_type {
        object.key("taxRegistrationNumberType").string(var_1.as_str());
    }
    if let Some(var_2) = &input.ppn_exception_designation_code {
        object.key("ppnExceptionDesignationCode").string(var_2.as_str());
    }
    if let Some(var_3) = &input.decision_number {
        object.key("decisionNumber").string(var_3.as_str());
    }
    Ok(())
}

pub(crate) fn de_indonesia_additional_info<'a, I>(
    tokens: &mut ::std::iter::Peekable<I>,
) -> ::std::result::Result<Option<crate::types::IndonesiaAdditionalInfo>, ::aws_smithy_json::deserialize::error::DeserializeError>
where
    I: Iterator<Item = Result<::aws_smithy_json::deserialize::Token<'a>, ::aws_smithy_json::deserialize::error::DeserializeError>>,
{
    match tokens.next().transpose()? {
        Some(::aws_smithy_json::deserialize::Token::ValueNull { .. }) => Ok(None),
        Some(::aws_smithy_json::deserialize::Token::StartObject { .. }) => {
            #[allow(unused_mut)]
            let mut builder = crate::types::builders::IndonesiaAdditionalInfoBuilder::default();
            loop {
                match tokens.next().transpose()? {
                    Some(::aws_smithy_json::deserialize::Token::EndObject { .. }) => break,
                    Some(::aws_smithy_json::deserialize::Token::ObjectKey { key, .. }) => match key.to_unescaped()?.as_ref() {
                        "taxRegistrationNumberType" => {
                            builder = builder.set_tax_registration_number_type(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| {
                                        s.to_unescaped()
                                            .map(|u| crate::types::IndonesiaTaxRegistrationNumberType::from(u.as_ref()))
                                    })
                                    .transpose()?,
                            );
                        }
                        "ppnExceptionDesignationCode" => {
                            builder = builder.set_ppn_exception_designation_code(
                                ::aws_smithy_json::deserialize::token::expect_string_or_null(tokens.next())?
                                    .map(|s| s.to_unescaped().map(|u| u.into_owned()))
                                    .transpose()?,
                            );
                        }
                        "decisionNumber" => {
                            builder = builder.set_decision_number(
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

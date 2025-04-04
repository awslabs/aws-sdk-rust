// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_stop_cis_message_progress(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::StopCisMessageProgress,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if input.total_checks != 0 {
        object.key("totalChecks").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.total_checks).into()),
        );
    }
    if input.successful_checks != 0 {
        object.key("successfulChecks").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.successful_checks).into()),
        );
    }
    if input.failed_checks != 0 {
        object.key("failedChecks").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.failed_checks).into()),
        );
    }
    if input.not_evaluated_checks != 0 {
        object.key("notEvaluatedChecks").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.not_evaluated_checks).into()),
        );
    }
    if input.unknown_checks != 0 {
        object.key("unknownChecks").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.unknown_checks).into()),
        );
    }
    if input.not_applicable_checks != 0 {
        object.key("notApplicableChecks").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.not_applicable_checks).into()),
        );
    }
    if input.informational_checks != 0 {
        object.key("informationalChecks").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.informational_checks).into()),
        );
    }
    if input.error_checks != 0 {
        object.key("errorChecks").number(
            #[allow(clippy::useless_conversion)]
            ::aws_smithy_types::Number::NegInt((input.error_checks).into()),
        );
    }
    Ok(())
}

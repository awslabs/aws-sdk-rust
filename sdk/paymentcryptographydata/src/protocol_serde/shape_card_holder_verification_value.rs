// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_card_holder_verification_value(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::CardHolderVerificationValue,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("UnpredictableNumber").string(input.unpredictable_number.as_str());
    }
    {
        object.key("PanSequenceNumber").string(input.pan_sequence_number.as_str());
    }
    {
        object
            .key("ApplicationTransactionCounter")
            .string(input.application_transaction_counter.as_str());
    }
    Ok(())
}

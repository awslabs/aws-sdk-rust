// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_delete_message_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::delete_message::DeleteMessageInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.queue_url {
        object.key("QueueUrl").string(var_1.as_str());
    }
    if let Some(var_2) = &input.receipt_handle {
        object.key("ReceiptHandle").string(var_2.as_str());
    }
    Ok(())
}

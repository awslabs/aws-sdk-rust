// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_stop_delivery_stream_encryption_input_input(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::operation::stop_delivery_stream_encryption::StopDeliveryStreamEncryptionInput,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    if let Some(var_1) = &input.delivery_stream_name {
        object.key("DeliveryStreamName").string(var_1.as_str());
    }
    Ok(())
}

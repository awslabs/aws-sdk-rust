// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn ser_fetch_page_request(
    object: &mut ::aws_smithy_json::serialize::JsonObjectWriter,
    input: &crate::types::FetchPageRequest,
) -> ::std::result::Result<(), ::aws_smithy_types::error::operation::SerializationError> {
    {
        object.key("TransactionId").string(input.transaction_id.as_str());
    }
    {
        object.key("NextPageToken").string(input.next_page_token.as_str());
    }
    Ok(())
}

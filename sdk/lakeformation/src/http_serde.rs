// Code generated by software.amazon.smithy.rust.codegen.smithy-rs. DO NOT EDIT.
pub fn deser_payload_get_work_unit_results_get_work_unit_results_output_result_stream(
    body: &mut aws_smithy_http::body::SdkBody,
) -> std::result::Result<
    aws_smithy_http::byte_stream::ByteStream,
    crate::error::GetWorkUnitResultsError,
> {
    // replace the body with an empty body
    let body = std::mem::replace(body, aws_smithy_http::body::SdkBody::taken());
    Ok(aws_smithy_http::byte_stream::ByteStream::new(body))
}

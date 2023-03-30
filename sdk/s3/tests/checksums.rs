/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_config::SdkConfig;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_http::user_agent::AwsUserAgent;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::Client;
use aws_sdk_s3::{operation::get_object::GetObjectOutput, types::ChecksumAlgorithm};
use aws_smithy_client::test_connection::{capture_request, TestConnection};
use aws_smithy_http::body::SdkBody;
use http::header::AUTHORIZATION;
use http::{HeaderValue, Uri};
use std::{
    convert::Infallible,
    time::{Duration, UNIX_EPOCH},
};

/// Test connection for the movies IT
/// headers are signed with actual creds, at some point we could replace them with verifiable test
/// credentials, but there are plenty of other tests that target signing
fn new_checksum_validated_response_test_connection(
    checksum_header_name: &'static str,
    checksum_header_value: &'static str,
) -> TestConnection<&'static str> {
    TestConnection::new(vec![
        (http::Request::builder()
             .header("x-amz-checksum-mode", "ENABLED")
             .header("user-agent", "aws-sdk-rust/0.123.test os/windows/XPSP3 lang/rust/1.50.0")
             .header("x-amz-date", "20210618T170728Z")
             .header("x-amz-content-sha256", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
             .header("x-amz-user-agent", "aws-sdk-rust/0.123.test api/test-service/0.123 os/windows/XPSP3 lang/rust/1.50.0")
             .header("authorization", "AWS4-HMAC-SHA256 Credential=ANOTREAL/20210618/us-east-1/s3/aws4_request, SignedHeaders=host;x-amz-checksum-mode;x-amz-content-sha256;x-amz-date;x-amz-security-token;x-amz-user-agent, Signature=eb9e58fa4fb04c8e6f160705017fdbb497ccff0efee4227b3a56f900006c3882")
             .uri(Uri::from_static("https://some-test-bucket.s3.us-east-1.amazonaws.com/test.txt?x-id=GetObject")).body(SdkBody::empty()).unwrap(),
         http::Response::builder()
             .header("x-amz-request-id", "4B4NGF0EAWN0GE63")
             .header("content-length", "11")
             .header("etag", "\"3e25960a79dbc69b674cd4ec67a72c62\"")
             .header(checksum_header_name, checksum_header_value)
             .header("content-type", "application/octet-stream")
             .header("server", "AmazonS3")
             .header("content-encoding", "")
             .header("last-modified", "Tue, 21 Jun 2022 16:29:14 GMT")
             .header("date", "Tue, 21 Jun 2022 16:29:23 GMT")
             .header("x-amz-id-2", "kPl+IVVZAwsN8ePUyQJZ40WD9dzaqtr4eNESArqE68GSKtVvuvCTDe+SxhTT+JTUqXB1HL4OxNM=")
             .header("accept-ranges", "bytes")
             .status(http::StatusCode::from_u16(200).unwrap())
             .body(r#"Hello world"#).unwrap()),
    ])
}

async fn test_checksum_on_streaming_response(
    checksum_header_name: &'static str,
    checksum_header_value: &'static str,
) -> GetObjectOutput {
    let conn = new_checksum_validated_response_test_connection(
        checksum_header_name,
        checksum_header_value,
    );
    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .region(Region::new("us-east-1"))
        .http_connector(conn.clone())
        .build();

    let client = Client::new(&sdk_config);

    let res = client
        .get_object()
        .bucket("some-test-bucket")
        .key("test.txt")
        .checksum_mode(aws_sdk_s3::types::ChecksumMode::Enabled)
        .customize()
        .await
        .unwrap()
        .map_operation(|mut op| {
            op.properties_mut()
                .insert(UNIX_EPOCH + Duration::from_secs(1624036048));
            op.properties_mut().insert(AwsUserAgent::for_tests());

            Result::Ok::<_, Infallible>(op)
        })
        .unwrap()
        .send()
        .await
        .unwrap();

    conn.assert_requests_match(&[
        http::header::HeaderName::from_static("x-amz-checksum-mode"),
        AUTHORIZATION,
    ]);

    res
}

#[tokio::test]
async fn test_crc32_checksum_on_streaming_response() {
    let res = test_checksum_on_streaming_response("x-amz-checksum-crc32", "i9aeUg==").await;

    // Header checksums are base64 encoded
    assert_eq!(res.checksum_crc32(), Some("i9aeUg=="));
    let body = collect_body_into_string(res.body.into_inner()).await;

    assert_eq!(body, "Hello world");
}

#[tokio::test]
async fn test_crc32c_checksum_on_streaming_response() {
    let res = test_checksum_on_streaming_response("x-amz-checksum-crc32c", "crUfeA==").await;

    // Header checksums are base64 encoded
    assert_eq!(res.checksum_crc32_c(), Some("crUfeA=="));
    let body = collect_body_into_string(res.body.into_inner()).await;

    assert_eq!(body, "Hello world");
}

#[tokio::test]
async fn test_sha1_checksum_on_streaming_response() {
    let res =
        test_checksum_on_streaming_response("x-amz-checksum-sha1", "e1AsOh9IyGCa4hLN+2Od7jlnP14=")
            .await;

    // Header checksums are base64 encoded
    assert_eq!(res.checksum_sha1(), Some("e1AsOh9IyGCa4hLN+2Od7jlnP14="));
    let body = collect_body_into_string(res.body.into_inner()).await;

    assert_eq!(body, "Hello world");
}

#[tokio::test]
async fn test_sha256_checksum_on_streaming_response() {
    let res = test_checksum_on_streaming_response(
        "x-amz-checksum-sha256",
        "ZOyIygCyaOW6GjVnihtTFtIS9PNmskdyMlNKiuyjfzw=",
    )
    .await;

    // Header checksums are base64 encoded
    assert_eq!(
        res.checksum_sha256(),
        Some("ZOyIygCyaOW6GjVnihtTFtIS9PNmskdyMlNKiuyjfzw=")
    );
    let body = collect_body_into_string(res.body.into_inner()).await;

    assert_eq!(body, "Hello world");
}

// The test structure is identical for all supported checksum algorithms
async fn test_checksum_on_streaming_request<'a>(
    body: &'static [u8],
    checksum_algorithm: ChecksumAlgorithm,
    checksum_header_name: &'static str,
    expected_decoded_content_length: &'a str,
    expected_encoded_content_length: &'a str,
    expected_aws_chunked_encoded_body: &'a str,
) {
    let (conn, rcvr) = capture_request(None);
    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .region(Region::new("us-east-1"))
        .http_connector(conn.clone())
        .build();

    let client = Client::new(&sdk_config);

    // ByteStreams created from a file are streaming and have a known size
    let mut file = tempfile::NamedTempFile::new().unwrap();
    use std::io::Write;
    file.write_all(body).unwrap();

    let body = aws_sdk_s3::primitives::ByteStream::read_from()
        .path(file.path())
        .buffer_size(1024)
        .build()
        .await
        .unwrap();

    // The response from the fake connection won't return the expected XML but we don't care about
    // that error in this test
    let _ = client
        .put_object()
        .bucket("test-bucket")
        .key("test.txt")
        .body(body)
        .checksum_algorithm(checksum_algorithm)
        .customize()
        .await
        .unwrap()
        .map_operation(|mut op| {
            op.properties_mut()
                .insert(UNIX_EPOCH + Duration::from_secs(1624036048));
            op.properties_mut().insert(AwsUserAgent::for_tests());

            Result::Ok::<_, Infallible>(op)
        })
        .unwrap()
        .send()
        .await
        .unwrap();

    let req = rcvr.expect_request();

    let headers = req.headers();
    let x_amz_content_sha256 = headers
        .get("x-amz-content-sha256")
        .expect("x-amz-content-sha256 header exists");
    let x_amz_trailer = headers
        .get("x-amz-trailer")
        .expect("x-amz-trailer header exists");
    let x_amz_decoded_content_length = headers
        .get("x-amz-decoded-content-length")
        .expect("x-amz-decoded-content-length header exists");
    let content_length = headers
        .get("Content-Length")
        .expect("Content-Length header exists");
    let content_encoding = headers
        .get("Content-Encoding")
        .expect("Content-Encoding header exists");

    assert_eq!(
        HeaderValue::from_static("STREAMING-UNSIGNED-PAYLOAD-TRAILER"),
        x_amz_content_sha256,
        "signing header is incorrect"
    );
    assert_eq!(
        HeaderValue::from_static(checksum_header_name),
        x_amz_trailer,
        "x-amz-trailer is incorrect"
    );
    assert_eq!(
        HeaderValue::from_static(aws_http::content_encoding::header_value::AWS_CHUNKED),
        content_encoding,
        "content-encoding wasn't set to aws-chunked"
    );

    // The length of the string "Hello world"
    assert_eq!(
        HeaderValue::from_str(expected_decoded_content_length).unwrap(),
        x_amz_decoded_content_length,
        "decoded content length was wrong"
    );
    // The sum of the length of the original body, chunk markers, and trailers
    assert_eq!(
        HeaderValue::from_str(expected_encoded_content_length).unwrap(),
        content_length,
        "content-length was expected to be {} but was {} instead",
        expected_encoded_content_length,
        content_length.to_str().unwrap()
    );

    let body = collect_body_into_string(req.into_body()).await;
    // When sending a streaming body with a checksum, the trailers are included as part of the body content
    assert_eq!(body.as_str(), expected_aws_chunked_encoded_body,);
}

#[tokio::test]
async fn test_crc32_checksum_on_streaming_request() {
    let expected_aws_chunked_encoded_body =
        "B\r\nHello world\r\n0\r\nx-amz-checksum-crc32:i9aeUg==\r\n\r\n";
    let expected_encoded_content_length = format!("{}", expected_aws_chunked_encoded_body.len());
    test_checksum_on_streaming_request(
        b"Hello world",
        ChecksumAlgorithm::Crc32,
        "x-amz-checksum-crc32",
        "11",
        &expected_encoded_content_length,
        expected_aws_chunked_encoded_body,
    )
    .await
}

// This test isn't a duplicate. It tests CRC32C (note the C) checksum request validation
#[tokio::test]
async fn test_crc32c_checksum_on_streaming_request() {
    let expected_aws_chunked_encoded_body =
        "B\r\nHello world\r\n0\r\nx-amz-checksum-crc32c:crUfeA==\r\n\r\n";
    let expected_encoded_content_length = format!("{}", expected_aws_chunked_encoded_body.len());
    test_checksum_on_streaming_request(
        b"Hello world",
        ChecksumAlgorithm::Crc32C,
        "x-amz-checksum-crc32c",
        "11",
        &expected_encoded_content_length,
        expected_aws_chunked_encoded_body,
    )
    .await
}

#[tokio::test]
async fn test_sha1_checksum_on_streaming_request() {
    let expected_aws_chunked_encoded_body =
        "B\r\nHello world\r\n0\r\nx-amz-checksum-sha1:e1AsOh9IyGCa4hLN+2Od7jlnP14=\r\n\r\n";
    let expected_encoded_content_length = format!("{}", expected_aws_chunked_encoded_body.len());
    test_checksum_on_streaming_request(
        b"Hello world",
        ChecksumAlgorithm::Sha1,
        "x-amz-checksum-sha1",
        "11",
        &expected_encoded_content_length,
        expected_aws_chunked_encoded_body,
    )
    .await
}

#[tokio::test]
async fn test_sha256_checksum_on_streaming_request() {
    let expected_aws_chunked_encoded_body = "B\r\nHello world\r\n0\r\nx-amz-checksum-sha256:ZOyIygCyaOW6GjVnihtTFtIS9PNmskdyMlNKiuyjfzw=\r\n\r\n";
    let expected_encoded_content_length = format!("{}", expected_aws_chunked_encoded_body.len());
    test_checksum_on_streaming_request(
        b"Hello world",
        ChecksumAlgorithm::Sha256,
        "x-amz-checksum-sha256",
        "11",
        &expected_encoded_content_length,
        expected_aws_chunked_encoded_body,
    )
    .await
}

async fn collect_body_into_string(mut body: aws_smithy_http::body::SdkBody) -> String {
    use bytes::Buf;
    use bytes_utils::SegmentedBuf;
    use http_body::Body;
    use std::io::Read;

    let mut output = SegmentedBuf::new();
    while let Some(buf) = body.data().await {
        output.push(buf.unwrap());
    }

    let mut output_text = String::new();
    output
        .reader()
        .read_to_string(&mut output_text)
        .expect("Doesn't cause IO errors");

    output_text
}

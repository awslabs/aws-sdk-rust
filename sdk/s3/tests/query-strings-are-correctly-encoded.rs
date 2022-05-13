/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_http::user_agent::AwsUserAgent;
use aws_sdk_s3::middleware::DefaultMiddleware;
use aws_sdk_s3::operation::ListObjectsV2;
use aws_sdk_s3::{Credentials, Region};
use aws_smithy_client::test_connection::capture_request;
use aws_smithy_client::Client as CoreClient;
use std::time::{Duration, UNIX_EPOCH};

pub type Client<C> = CoreClient<C, DefaultMiddleware>;

#[tokio::test]
async fn test_s3_signer_query_string_with_all_valid_chars() -> Result<(), aws_sdk_s3::Error> {
    let creds = Credentials::new(
        "ANOTREAL",
        "notrealrnrELgWzOk3IfjzDKtFBhDby",
        Some("notarealsessiontoken".to_string()),
        None,
        "test",
    );
    let conf = aws_sdk_s3::Config::builder()
        .credentials_provider(creds)
        .region(Region::new("us-east-1"))
        .build();
    let (conn, rcvr) = capture_request(None);

    let client = Client::new(conn.clone());

    // Generate a string containing all printable ASCII chars
    let prefix: String = (32u8..127).map(char::from).collect();

    let mut op = ListObjectsV2::builder()
        .bucket("test-bucket")
        .prefix(&prefix)
        .build()
        .unwrap()
        .make_operation(&conf)
        .await
        .expect("failed to construct operation");
    op.properties_mut()
        .insert(UNIX_EPOCH + Duration::from_secs(1624036048));
    op.properties_mut().insert(AwsUserAgent::for_tests());

    // The response from the fake connection won't return the expected XML but we don't care about
    // that error in this test
    let _ = client.call(op).await;

    let expected_req = rcvr.expect_request();
    let auth_header = expected_req
        .headers()
        .get("Authorization")
        .unwrap()
        .to_owned();

    // This is a snapshot test taken from a known working test result
    let snapshot_signature =
        "Signature=775f88213304a5233ff295f869571554140e3db171a2d4a64f63902c49f79880";
    assert!(
        auth_header
            .to_str()
            .unwrap()
            .contains(snapshot_signature),
        "authorization header signature did not match expected signature: got {}, expected it to contain {}",
        auth_header.to_str().unwrap(),
        snapshot_signature
    );

    Ok(())
}

// This test can help identify individual characters that break the signing of query strings. This
// test must be run against an actual bucket so we `ignore` it unless the runner specifically requests it
#[tokio::test]
#[ignore]
async fn test_query_strings_are_correctly_encoded() -> Result<(), aws_sdk_s3::Error> {
    use aws_sdk_s3::error::{ListObjectsV2Error, ListObjectsV2ErrorKind};
    use aws_smithy_http::result::SdkError;

    tracing_subscriber::fmt::init();
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);

    let mut chars_that_break_signing = Vec::new();
    let mut chars_that_break_uri_parsing = Vec::new();
    let mut chars_that_are_invalid_arguments = Vec::new();

    // We test all possible bytes to check for issues with URL construction or signing
    for byte in u8::MIN..u8::MAX {
        let char = char::from(byte);
        let res = client
            .list_objects_v2()
            .bucket("a-bucket-to-test-with")
            .prefix(char)
            .send()
            .await;
        if let Err(SdkError::ServiceError {
            err: ListObjectsV2Error { kind, .. },
            ..
        }) = res
        {
            match kind {
                ListObjectsV2ErrorKind::Unhandled(e)
                    if e.to_string().contains("SignatureDoesNotMatch") =>
                {
                    chars_that_break_signing.push(byte);
                }
                ListObjectsV2ErrorKind::Unhandled(e) if e.to_string().contains("InvalidUri") => {
                    chars_that_break_uri_parsing.push(byte);
                }
                ListObjectsV2ErrorKind::Unhandled(e)
                    if e.to_string().contains("InvalidArgument") =>
                {
                    chars_that_are_invalid_arguments.push(byte);
                }
                ListObjectsV2ErrorKind::Unhandled(e) if e.to_string().contains("InvalidToken") => {
                    panic!("refresh your credentials and run this test again");
                }
                e => todo!("unexpected error: {:?}", e),
            }
        }
    }

    if chars_that_break_signing.is_empty()
        && chars_that_break_uri_parsing.is_empty()
        && chars_that_are_invalid_arguments.is_empty()
    {
        Ok(())
    } else {
        fn char_transform(c: u8) -> String {
            format!("byte {}: {}\n", c, char::from(c))
        }
        if !chars_that_break_signing.is_empty() {
            tracing::error!(
                "The following characters caused a signature mismatch:\n{}(end)",
                chars_that_break_signing
                    .clone()
                    .into_iter()
                    .map(char_transform)
                    .collect::<String>()
            );
        }
        if !chars_that_break_uri_parsing.is_empty() {
            tracing::error!(
                "The following characters caused a URI parse failure:\n{}(end)",
                chars_that_break_uri_parsing
                    .clone()
                    .into_iter()
                    .map(char_transform)
                    .collect::<String>()
            );
        }
        if !chars_that_are_invalid_arguments.is_empty() {
            tracing::error!(
                "The following characters caused an \"Invalid Argument\" failure:\n{}(end)",
                chars_that_are_invalid_arguments
                    .clone()
                    .into_iter()
                    .map(char_transform)
                    .collect::<String>()
            );
        }
        panic!("test failed, see logs for the problem chars")
    }
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_http::user_agent::AwsUserAgent;
use aws_sdk_s3::{config::Credentials, config::Region, primitives::ByteStream, Client};
use aws_smithy_client::test_connection::capture_request;
use aws_types::SdkConfig;
use http::HeaderValue;
use std::{
    convert::Infallible,
    time::{Duration, UNIX_EPOCH},
};

const NAUGHTY_STRINGS: &str = include_str!("blns/blns.txt");

// // A useful way to find leaks in the signing system that requires an actual S3 bucket to test with
// // If you want to use this, update the credentials to be your credentials and change the bucket name
// // to your bucket
// // NOTE: this won't actually succeed, you'll get a 400 back from S3 because the headers are too long.
// #[tokio::test]
// async fn test_metadata_field_against_naughty_strings_list() -> Result<(), aws_sdk_s3::Error> {
//     // re-add `aws-config = { path = "../../build/aws-sdk/aws-config" }` to this project's Cargo.toml
//
//     let config = aws_config::load_from_env().await;
//     let client = aws_sdk_s3::Client::new(&config);
//
//     let mut req = client
//         .put_object()
//         .bucket("your-test-bucket-goes-here")
//         .key("test.txt")
//         .body(aws_sdk_s3::ByteStream::from_static(b"some test text"));
//
//     for (idx, line) in NAUGHTY_STRINGS.split('\n').enumerate() {
//         // add lines to metadata unless they're a comment or empty
//         // Some naughty strings aren't valid HeaderValues so we skip those too
//         if !line.starts_with("#") && !line.is_empty() && HeaderValue::from_str(line).is_ok() {
//             let key = format!("line-{}", idx);
//
//             req = req.metadata(key, line);
//         }
//     }
//
//     // If this fails due to signing then the signer choked on a bad string. To find out which string,
//     // send one request per line instead of adding all lines as metadata for one request.
//     let _ = req.send().await.unwrap();
//
//     Ok(())
// }

#[tokio::test]
async fn test_s3_signer_with_naughty_string_metadata() {
    let (conn, rcvr) = capture_request(None);
    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .region(Region::new("us-east-1"))
        .http_connector(conn.clone())
        .build();
    let config = aws_sdk_s3::config::Builder::from(&sdk_config)
        .force_path_style(true)
        .build();

    let client = Client::from_conf(config);
    let mut builder = client
        .put_object()
        .bucket("test-bucket")
        .key("text.txt")
        .body(ByteStream::from_static(b"some test text"));

    for (idx, line) in NAUGHTY_STRINGS.split('\n').enumerate() {
        // add lines to metadata unless they're a comment or empty
        // Some naughty strings aren't valid HeaderValues so we skip those too
        if !line.starts_with('#') && !line.is_empty() && HeaderValue::from_str(line).is_ok() {
            let key = format!("line-{}", idx);

            builder = builder.metadata(key, line);
        }
    }

    let _ = builder
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

    let expected_req = rcvr.expect_request();
    let auth_header = expected_req
        .headers()
        .get("Authorization")
        .unwrap()
        .to_owned();

    // This is a snapshot test taken from a known working test result
    let snapshot_signature =
        "Signature=8dfa41f2db599a9fba53393b0ae5da646e5e452fa3685f7a1487d6eade5ec5c8";
    assert!(
        auth_header
            .to_str()
            .unwrap()
            .contains(snapshot_signature),
        "authorization header signature did not match expected signature: got {}, expected it to contain {}",
        auth_header.to_str().unwrap(),
        snapshot_signature
    );
}

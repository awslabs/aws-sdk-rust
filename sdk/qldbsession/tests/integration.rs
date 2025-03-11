/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg(feature = "test-util")]

use aws_sdk_qldbsession::config::{Config, Credentials, Region};
use aws_sdk_qldbsession::types::StartSessionRequest;
use aws_sdk_qldbsession::Client;
use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::body::SdkBody;
use http_1x::Uri;

#[cfg(feature = "test-util")]
#[tokio::test]
async fn signv4_use_correct_service_name() {
    let http_client = StaticReplayClient::new(vec![ReplayEvent::new(
        http_1x::Request::builder()
            .header("content-type", "application/x-amz-json-1.0")
            .header("x-amz-target", "QLDBSession.SendCommand")
            .header("content-length", "49")
            .header("authorization", "AWS4-HMAC-SHA256 Credential=ANOTREAL/20090213/us-east-1/qldb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-date;x-amz-target;x-amz-user-agent, Signature=9a07c60550504d015fb9a2b0f1b175a4d906651f9dd4ee44bebb32a802d03815")
            // qldbsession uses the signing name 'qldb' in signature _________________________^^^^
            .header("x-amz-date", "20090213T233130Z")
            .header("user-agent", "aws-sdk-rust/0.123.test os/windows/XPSP3 lang/rust/1.50.0")
            .uri(Uri::from_static("https://session.qldb.us-east-1.amazonaws.com/"))
            .body(SdkBody::from(r#"{"StartSession":{"LedgerName":"not-real-ledger"}}"#)).unwrap(),
        http_1x::Response::builder()
            .status(http_1x::StatusCode::from_u16(200).unwrap())
            .body(SdkBody::from(r#"{}"#)).unwrap()),
    ]);
    let conf = Config::builder()
        .http_client(http_client.clone())
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests_with_session_token())
        .with_test_defaults()
        .build();
    let client = Client::from_conf(conf);

    let _ = client
        .send_command()
        .start_session(
            StartSessionRequest::builder()
                .ledger_name("not-real-ledger")
                .build()
                .unwrap(),
        )
        .customize()
        .mutate_request(|req| {
            // Remove the invocation ID since the signed request above doesn't have it
            req.headers_mut().remove("amz-sdk-invocation-id");
        })
        .send()
        .await
        .expect("request should succeed");

    http_client.assert_requests_match(&["authorization"]);
}

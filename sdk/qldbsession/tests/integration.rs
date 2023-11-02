/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_qldbsession::config::{Config, Credentials, Region};
use aws_sdk_qldbsession::types::StartSessionRequest;
use aws_sdk_qldbsession::Client;
use aws_smithy_http::body::SdkBody;
use aws_smithy_runtime::client::http::test_util::{ReplayEvent, StaticReplayClient};
use http::Uri;
use std::time::{Duration, UNIX_EPOCH};

#[tokio::test]
async fn signv4_use_correct_service_name() {
    let http_client = StaticReplayClient::new(vec![ReplayEvent::new(
        http::Request::builder()
            .header("content-type", "application/x-amz-json-1.0")
            .header("x-amz-target", "QLDBSession.SendCommand")
            .header("content-length", "49")
            .header("authorization", "AWS4-HMAC-SHA256 Credential=ANOTREAL/20210305/us-east-1/qldb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-date;x-amz-security-token;x-amz-target;x-amz-user-agent, Signature=350f957e9b736ac3f636d16c59c0a3cee8c2780b0ffadc99bbca841b7f15bee4")
            // qldbsession uses the signing name 'qldb' in signature _________________________^^^^
            .header("x-amz-date", "20210305T134922Z")
            .header("x-amz-security-token", "notarealsessiontoken")
            .header("user-agent", "aws-sdk-rust/0.123.test os/windows/XPSP3 lang/rust/1.50.0")
            .uri(Uri::from_static("https://session.qldb.us-east-1.amazonaws.com/"))
            .body(SdkBody::from(r#"{"StartSession":{"LedgerName":"not-real-ledger"}}"#)).unwrap(),
        http::Response::builder()
            .status(http::StatusCode::from_u16(200).unwrap())
            .body(SdkBody::from(r#"{}"#)).unwrap()),
    ]);
    let conf = Config::builder()
        .http_client(http_client.clone())
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests_with_session_token())
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
        .await
        .expect("should be customizable")
        // Fix the request time and user agent so the headers are stable
        .request_time_for_tests(UNIX_EPOCH + Duration::from_secs(1614952162))
        .user_agent_for_tests()
        .mutate_request(|req| {
            // Remove the invocation ID since the signed request above doesn't have it
            req.headers_mut().remove("amz-sdk-invocation-id");
        })
        .send()
        .await
        .expect("request should succeed");

    http_client.assert_requests_match(&[]);
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_qldbsession as qldbsession;
use aws_smithy_client::test_connection::TestConnection;
use aws_smithy_http::body::SdkBody;
use http::Uri;
use qldbsession::config::{Config, Credentials, Region};
use qldbsession::types::StartSessionRequest;
use qldbsession::Client;
use std::time::{Duration, UNIX_EPOCH};

// TODO(DVR): having the full HTTP requests right in the code is a bit gross, consider something
// like https://github.com/davidbarsky/sigv4/blob/master/aws-sigv4/src/lib.rs#L283-L315 to store
// the requests/responses externally

#[tokio::test]
async fn signv4_use_correct_service_name() {
    let conn = TestConnection::new(vec![(
        http::Request::builder()
            .header("content-type", "application/x-amz-json-1.0")
            .header("x-amz-target", "QLDBSession.SendCommand")
            .header("content-length", "49")
            .header("authorization", "AWS4-HMAC-SHA256 Credential=ANOTREAL/20210305/us-east-1/qldb/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-date;x-amz-security-token;x-amz-target;x-amz-user-agent, Signature=350f957e9b736ac3f636d16c59c0a3cee8c2780b0ffadc99bbca841b7f15bee4")
            // qldbsession uses the service name 'qldb' in signature ____________________________________^^^^
            .header("x-amz-date", "20210305T134922Z")
            .header("x-amz-security-token", "notarealsessiontoken")
            .header("user-agent", "aws-sdk-rust/0.123.test os/windows/XPSP3 lang/rust/1.50.0")
            .uri(Uri::from_static("https://session.qldb.us-east-1.amazonaws.com/"))
            .body(SdkBody::from(r#"{"StartSession":{"LedgerName":"not-real-ledger"}}"#)).unwrap(),
        http::Response::builder()
            .status(http::StatusCode::from_u16(200).unwrap())
            .body(r#"{}"#).unwrap()),
    ]);
    let conf = Config::builder()
        .http_connector(conn.clone())
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .build();
    let client = Client::from_conf(conf);

    let _ = client
        .send_command()
        .start_session(
            StartSessionRequest::builder()
                .ledger_name("not-real-ledger")
                .build(),
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

    conn.assert_requests_match(&[]);
}

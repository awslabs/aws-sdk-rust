/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_http::user_agent::AwsUserAgent;
use aws_sdk_kms as kms;
use aws_sdk_kms::middleware::DefaultMiddleware;
use aws_sdk_kms::operation::RequestId;
use aws_smithy_client::test_connection::TestConnection;
use aws_smithy_client::{Client as CoreClient, SdkError};
use aws_smithy_http::body::SdkBody;
use http::header::AUTHORIZATION;
use http::Uri;
use kms::config::{Config, Credentials, Region};
use kms::operation::generate_random::GenerateRandomInput;
use std::time::{Duration, UNIX_EPOCH};

type Client<C> = CoreClient<C, DefaultMiddleware>;

// TODO(DVR): having the full HTTP requests right in the code is a bit gross, consider something
// like https://github.com/davidbarsky/sigv4/blob/master/aws-sigv4/src/lib.rs#L283-L315 to store
// the requests/responses externally

/// Validate that for CN regions we set the URI correctly
#[tokio::test]
async fn generate_random_cn() {
    let conn = TestConnection::new(vec![(
        http::Request::builder()
            .uri(Uri::from_static("https://kms.cn-north-1.amazonaws.com.cn/"))
            .body(SdkBody::from(r#"{"NumberOfBytes":64}"#)).unwrap(),
        http::Response::builder()
            .status(http::StatusCode::from_u16(200).unwrap())
            .body(r#"{"Plaintext":"6CG0fbzzhg5G2VcFCPmJMJ8Njv3voYCgrGlp3+BZe7eDweCXgiyDH9BnkKvLmS7gQhnYDUlyES3fZVGwv5+CxA=="}"#).unwrap())
    ]);
    let conf = Config::builder()
        .region(Region::new("cn-north-1"))
        .credentials_provider(Credentials::for_tests())
        .http_connector(conn.clone())
        .build();
    let client = kms::Client::from_conf(conf);
    let _ = client
        .generate_random()
        .number_of_bytes(64)
        .send()
        .await
        .expect("success");

    assert_eq!(conn.requests().len(), 1);
    conn.assert_requests_match(&[]);
}

#[tokio::test]
async fn generate_random() {
    let conn = TestConnection::new(vec![(
        http::Request::builder()
            .header("content-type", "application/x-amz-json-1.1")
            .header("x-amz-target", "TrentService.GenerateRandom")
            .header("content-length", "20")
            .header("authorization", "AWS4-HMAC-SHA256 Credential=ANOTREAL/20210305/us-east-1/kms/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-date;x-amz-security-token;x-amz-target;x-amz-user-agent, Signature=2e0dd7259fba92523d553173c452eba8a6ee7990fb5b1f8e2eccdeb75309e9e1")
            .header("x-amz-date", "20210305T134922Z")
            .header("x-amz-security-token", "notarealsessiontoken")
            .header("user-agent", "aws-sdk-rust/0.123.test os/windows/XPSP3 lang/rust/1.50.0")
            .header("x-amz-user-agent", "aws-sdk-rust/0.123.test api/test-service/0.123 os/windows/XPSP3 lang/rust/1.50.0")
            .uri(Uri::from_static("https://kms.us-east-1.amazonaws.com/"))
            .body(SdkBody::from(r#"{"NumberOfBytes":64}"#)).unwrap(),
        http::Response::builder()
            .status(http::StatusCode::from_u16(200).unwrap())
            .body(r#"{"Plaintext":"6CG0fbzzhg5G2VcFCPmJMJ8Njv3voYCgrGlp3+BZe7eDweCXgiyDH9BnkKvLmS7gQhnYDUlyES3fZVGwv5+CxA=="}"#).unwrap())
    ]);
    let client = Client::new(conn.clone());
    let conf = Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .build();
    let mut op = GenerateRandomInput::builder()
        .number_of_bytes(64)
        .build()
        .unwrap()
        .make_operation(&conf)
        .await
        .expect("valid operation");
    op.properties_mut()
        .insert(UNIX_EPOCH + Duration::from_secs(1614952162));
    op.properties_mut().insert(AwsUserAgent::for_tests());
    let resp = client.call(op).await.expect("request should succeed");
    // primitive checksum
    assert_eq!(
        resp.plaintext
            .expect("blob should exist")
            .as_ref()
            .iter()
            .map(|i| *i as u32)
            .sum::<u32>(),
        8562
    );
    conn.assert_requests_match(&[]);
}

#[tokio::test]
async fn generate_random_malformed_response() {
    let conn = TestConnection::new(vec![(
        http::Request::builder().body(SdkBody::from(r#"{"NumberOfBytes":64}"#)).unwrap(),
        http::Response::builder()
            .status(http::StatusCode::from_u16(200).unwrap())
            // last `}` replaced with a space, invalid JSON
            .body(r#"{"Plaintext":"6CG0fbzzhg5G2VcFCPmJMJ8Njv3voYCgrGlp3+BZe7eDweCXgiyDH9BnkKvLmS7gQhnYDUlyES3fZVGwv5+CxA==" "#).unwrap())
    ]);
    let client = Client::new(conn.clone());
    let conf = Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .build();
    let op = GenerateRandomInput::builder()
        .number_of_bytes(64)
        .build()
        .unwrap()
        .make_operation(&conf)
        .await
        .expect("valid operation");
    client.call(op).await.expect_err("response was malformed");
}

#[tokio::test]
async fn generate_random_keystore_not_found() {
    let conf = Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .build();
    let conn = TestConnection::new(vec![(
        http::Request::builder()
            .header("content-type", "application/x-amz-json-1.1")
            .header("x-amz-target", "TrentService.GenerateRandom")
            .header("content-length", "56")
            .header("authorization", "AWS4-HMAC-SHA256 Credential=ANOTREAL/20210305/us-east-1/kms/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-target, Signature=4ca5cde61676c0ee49fde9ba3c886967e8af16461b6aafdfaee18033eb4ac7a5")
            .header("x-amz-date", "20210305T144724Z")
            .header("x-amz-security-token", "notarealsessiontoken")
            .header("user-agent", "aws-sdk-rust/0.123.test os/windows/XPSP3 lang/rust/1.50.0")
            .header("x-amz-user-agent", "aws-sdk-rust/0.123.test api/test-service/0.123 os/windows/XPSP3 lang/rust/1.50.0")
            .uri(Uri::from_static("https://kms.us-east-1.amazonaws.com/"))
            .body(SdkBody::from(r#"{"NumberOfBytes":64,"CustomKeyStoreId":"does not exist"}"#)).unwrap(),
        http::Response::builder()
            .status(http::StatusCode::from_u16(400).unwrap())
            .header("x-amzn-requestid", "bfe81a0a-9a08-4e71-9910-cdb5ab6ea3b6")
            .header("cache-control", "no-cache, no-store, must-revalidate, private")
            .header("expires", "0")
            .header("pragma", "no-cache")
            .header("date", "Fri, 05 Mar 2021 15:01:40 GMT")
            .header("content-type", "application/x-amz-json-1.1")
            .header("content-length", "44")
            .body(r#"{"__type":"CustomKeyStoreNotFoundException"}"#).unwrap())
    ]);

    let mut op = GenerateRandomInput::builder()
        .number_of_bytes(64)
        .custom_key_store_id("does not exist")
        .build()
        .unwrap()
        .make_operation(&conf)
        .await
        .expect("valid operation");

    op.properties_mut()
        .insert(UNIX_EPOCH + Duration::from_secs(1614955644));
    op.properties_mut().insert(AwsUserAgent::for_tests());
    let client = Client::new(conn.clone());
    let err = client.call(op).await.expect_err("key store doesn't exist");
    let inner = match err {
        SdkError::ServiceError(context) => context.into_err(),
        other => panic!("Incorrect error received: {:}", other),
    };
    assert!(inner.is_custom_key_store_not_found_exception());
    assert_eq!(
        inner.request_id(),
        Some("bfe81a0a-9a08-4e71-9910-cdb5ab6ea3b6")
    );
    conn.assert_requests_match(&[AUTHORIZATION]);
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! End-to-end tests for the scoped, registry-backed error-dispatch fallback.
//!
//! When a response carries an error code an operation does not model directly,
//! the schema-serde error path reifies it against the operation's own error
//! registry first, then widens to the service-wide error registry. On a hit the
//! reified typed error is attached as the `source` of the returned `Unhandled`
//! error (the variant a caller receives is unchanged; only `Error::source()` is
//! enriched). On a miss the error stays generic.
//!
//! `DescribeTable` models only `InternalServerError`, `InvalidEndpointException`
//! and `ResourceNotFoundException`, so it is a convenient operation that does
//! *not* model `ConditionalCheckFailedException` — which is nonetheless present
//! in the service-wide error registry (it is modeled on `PutItem`).
//!
//! This behavior only exists on the schema-serde path, which is generated for
//! DynamoDB only while its protocol is on `SchemaSerdeAllowlist`. The tests
//! therefore compile and run only while that protocol is enabled (it currently
//! is not, the allowlist being empty, so they are **disabled**).
//!
//! Note these tests would still *compile* on the legacy path — they reference
//! only ordinary generated types — but the registry-backed `Error::source()`
//! enrichment they assert does not exist there, so they would fail rather than
//! fail to build.
//!
//! TODO(schema-serde): Rust cannot query the codegen allowlist, so this gating
//! is manual — remove the `#![cfg(any())]` below to re-enable these tests when
//! awsJson1_0 (or DynamoDB specifically) is re-added to the allowlist.

#![cfg(any())]

use aws_sdk_dynamodb::config::{
    BehaviorVersion, Credentials, Region, StalledStreamProtectionConfig,
};
use aws_sdk_dynamodb::error::ProvideErrorMetadata;
use aws_sdk_dynamodb::types::error::ConditionalCheckFailedException;
use aws_sdk_dynamodb::Client;
use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::body::SdkBody;

/// Build a client whose single response is an awsJson1.0 error with `body` and a
/// 400 status (a client error, so no retries fire).
fn client_returning_error(body: &'static str) -> Client {
    let http_client = StaticReplayClient::new(vec![ReplayEvent::new(
        http_1x::Request::builder()
            .uri("https://dynamodb.us-east-1.amazonaws.com/")
            .body(SdkBody::from(""))
            .unwrap(),
        http_1x::Response::builder()
            .status(400)
            .body(SdkBody::from(body))
            .unwrap(),
    )]);
    let config = aws_sdk_dynamodb::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .credentials_provider(Credentials::for_tests())
        .region(Region::new("us-east-1"))
        .http_client(http_client)
        .build();
    Client::from_conf(config)
}

#[tokio::test]
async fn tier2_reifies_unmodeled_service_error_as_source() {
    // `ConditionalCheckFailedException` is not a `DescribeTable` error, but it is
    // in the service-wide registry. The fallback reifies it and attaches it as
    // the source, without reclassifying it into a modeled `DescribeTable` variant.
    let body =
        r#"{"__type":"com.amazonaws.dynamodb#ConditionalCheckFailedException","message":"boom"}"#;
    let err = client_returning_error(body)
        .describe_table()
        .table_name("t")
        .send()
        .await
        .unwrap_err();
    let svc = err.into_service_error();

    // The error metadata (code/message) is preserved.
    assert_eq!(svc.code(), Some("ConditionalCheckFailedException"));

    // It is NOT reclassified into one of the operation's modeled variants — it
    // stays the catch-all (`Unhandled`) error.
    assert!(!svc.is_internal_server_error());
    assert!(!svc.is_invalid_endpoint_exception());
    assert!(!svc.is_resource_not_found_exception());

    // The reified typed error is attached as the `source` and downcasts.
    let source = std::error::Error::source(&svc).expect("unhandled error carries a source");
    let reified = source
        .downcast_ref::<ConditionalCheckFailedException>()
        .expect("source is the reified ConditionalCheckFailedException");
    assert_eq!(reified.message(), Some("boom"));
}

#[tokio::test]
async fn tier2_unknown_code_stays_generic() {
    // A code in neither the operation nor the service registry: the fallback
    // misses, so the error stays generic and no typed service error is attached.
    let body = r#"{"__type":"com.amazonaws.dynamodb#TotallyUnknownError","message":"boom"}"#;
    let err = client_returning_error(body)
        .describe_table()
        .table_name("t")
        .send()
        .await
        .unwrap_err();
    let svc = err.into_service_error();

    assert_eq!(svc.code(), Some("TotallyUnknownError"));

    // The source is the plain error metadata, not a reified service error.
    let source = std::error::Error::source(&svc).expect("unhandled error carries a source");
    assert!(
        source
            .downcast_ref::<ConditionalCheckFailedException>()
            .is_none(),
        "an unknown code must not reify into a typed service error"
    );
}

#[tokio::test]
async fn modeled_error_still_takes_tier1() {
    // A code the operation models is dispatched by the typed Tier-1 match and is
    // unaffected by the Tier-2 fallback.
    let body = r#"{"__type":"com.amazonaws.dynamodb#ResourceNotFoundException","message":"nope"}"#;
    let err = client_returning_error(body)
        .describe_table()
        .table_name("t")
        .send()
        .await
        .unwrap_err();
    let svc = err.into_service_error();

    assert!(svc.is_resource_not_found_exception());
    assert_eq!(svc.code(), Some("ResourceNotFoundException"));
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Tests that protocols can be swapped at runtime via the `protocol()` config setter.
//!
//! TODO(schema-serde): Re-enable these tests when schema-serde codegen is
//! active for DynamoDB (awsJson1_0). The runtime protocol swap feature
//! requires the generated `config::Builder::protocol(...)` setter, which is
//! only emitted by `SchemaProtocolConfigCustomization` when the service is on
//! `SchemaSerdeAllowlist`. With the allowlist empty on main, DynamoDB's
//! config builder does not expose `protocol(...)` and these tests cannot
//! compile. Once awsJson1_0 (or DynamoDB specifically) is re-added to the
//! allowlist, uncomment the block below.
//! See: codegen-client/.../customizations/SchemaDecorator.kt

// --- BEGIN schema-serde protocol-swap tests (disabled) ---
/*
use aws_sdk_dynamodb::config::{Credentials, Region, StalledStreamProtectionConfig};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_smithy_http_client::test_util::capture_request;

fn base_config() -> aws_sdk_dynamodb::config::Builder {
    aws_sdk_dynamodb::config::Builder::new()
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .endpoint_url("http://localhost:8000")
}

/// Default protocol (awsJson1_0) serializes with the correct Content-Type and X-Amz-Target.
#[tokio::test]
async fn default_protocol_serializes_correctly() {
    let (http_client, rx) = capture_request(None);
    let conf = base_config().http_client(http_client).build();
    let client = aws_sdk_dynamodb::Client::from_conf(conf);

    let _ = client
        .put_item()
        .table_name("TestTable")
        .item("pk", AttributeValue::S("key1".into()))
        .send()
        .await;

    let request = rx.expect_request();
    assert_eq!(
        request.headers().get("content-type").unwrap(),
        "application/x-amz-json-1.0",
        "default DynamoDB protocol should use awsJson1.0 content type"
    );
    assert!(
        request
            .headers()
            .get("x-amz-target")
            .unwrap()
            .contains("DynamoDB_20120810.PutItem"),
        "default protocol should set X-Amz-Target header"
    );

    // Verify the body is valid JSON with the expected structure
    let body = std::str::from_utf8(request.body().bytes().unwrap()).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(parsed["TableName"], "TestTable");
    assert!(parsed["Item"]["pk"]["S"].as_str().is_some());
}

/// Swapping to awsJson1_1 at runtime changes the Content-Type header.
#[tokio::test]
async fn swapped_protocol_changes_content_type() {
    let (http_client, rx) = capture_request(None);
    let custom_protocol = aws_smithy_json::protocol::aws_json_rpc::AwsJsonRpcProtocol::aws_json_1_1(
        "DynamoDB_20120810",
    );
    let conf = base_config()
        .http_client(http_client)
        .protocol(custom_protocol)
        .build();
    let client = aws_sdk_dynamodb::Client::from_conf(conf);

    let _ = client
        .put_item()
        .table_name("TestTable")
        .item("pk", AttributeValue::S("key1".into()))
        .send()
        .await;

    let request = rx.expect_request();
    assert_eq!(
        request.headers().get("content-type").unwrap(),
        "application/x-amz-json-1.1",
        "swapped protocol should use awsJson1.1 content type"
    );
    assert!(
        request
            .headers()
            .get("x-amz-target")
            .unwrap()
            .contains("DynamoDB_20120810.PutItem"),
        "swapped protocol should still set X-Amz-Target header"
    );

    // Body should still be valid JSON with the same structure
    let body = std::str::from_utf8(request.body().bytes().unwrap()).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(parsed["TableName"], "TestTable");
}

/// Swapping to a REST protocol (restJson1) on an RPC service still serializes
/// correctly. The restJson1 protocol returns `supports_http_bindings() == true`,
/// but since DynamoDB's operations have no HTTP binding traits, the generated
/// code takes the `serialize_request()` path regardless. This verifies that a
/// fundamentally different protocol class can be plugged in at runtime.
#[tokio::test]
async fn swap_to_rest_json_protocol() {
    let (http_client, rx) = capture_request(None);
    let rest_protocol = aws_smithy_json::protocol::aws_rest_json_1::AwsRestJsonProtocol::new();
    let conf = base_config()
        .http_client(http_client)
        .protocol(rest_protocol)
        .build();
    let client = aws_sdk_dynamodb::Client::from_conf(conf);

    let _ = client
        .put_item()
        .table_name("TestTable")
        .item("pk", AttributeValue::S("key1".into()))
        .send()
        .await;

    let request = rx.expect_request();

    // restJson1 uses application/json content type
    assert_eq!(
        request.headers().get("content-type").unwrap(),
        "application/json",
        "restJson1 protocol should use application/json content type"
    );

    // restJson1 does NOT set X-Amz-Target (that's an awsJson RPC thing)
    assert!(
        request.headers().get("x-amz-target").is_none(),
        "restJson1 protocol should not set X-Amz-Target header"
    );

    // Body should still be valid JSON with the expected members
    let body = std::str::from_utf8(request.body().bytes().unwrap()).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(parsed["TableName"], "TestTable");
    assert!(parsed["Item"]["pk"]["S"].as_str().is_some());
}
*/
// --- END schema-serde protocol-swap tests (disabled) ---

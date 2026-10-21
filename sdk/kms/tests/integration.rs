/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_kms as kms;
use aws_sdk_kms::operation::RequestId;
use aws_smithy_cbor::Encoder;
use aws_smithy_http_client::test_util::{capture_request, ReplayEvent, StaticReplayClient};
use aws_smithy_runtime_api::client::result::SdkError;
use aws_smithy_types::base64;
use aws_smithy_types::body::SdkBody;
use http_1x::Uri;
use kms::config::{Config, Credentials, Region};

// TODO(DVR): having the full HTTP requests right in the code is a bit gross, consider something
// like https://github.com/davidbarsky/sigv4/blob/master/aws-sigv4/src/lib.rs#L283-L315 to store
// the requests/responses externally

/// 64 bytes of plaintext whose bytes sum to 8562, the checksum `generate_random` asserts.
const PLAINTEXT_B64: &str =
    "6CG0fbzzhg5G2VcFCPmJMJ8Njv3voYCgrGlp3+BZe7eDweCXgiyDH9BnkKvLmS7gQhnYDUlyES3fZVGwv5+CxA==";

/// The wire protocol the generated client speaks.
///
/// This is only used in a transitional phase where the tests deal with more than one protocol.
// TODO(kmsWireFormat): delete the arm that no longer matches the model, and inline the bytes
// of the one that does.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Wire {
    AwsJson,
    RpcV2Cbor,
}

impl Wire {
    fn path(self) -> &'static str {
        match self {
            // awsJson1_1 posts every operation to the root and names it in `x-amz-target`.
            Wire::AwsJson => "/",
            // rpcv2Cbor carries the service and the operation in the path.
            Wire::RpcV2Cbor => "/service/TrentService/operation/GenerateRandom",
        }
    }

    fn content_type(self) -> &'static str {
        match self {
            Wire::AwsJson => "application/x-amz-json-1.1",
            Wire::RpcV2Cbor => "application/cbor",
        }
    }

    fn generate_random_input(self, custom_key_store_id: Option<&str>) -> Vec<u8> {
        match self {
            Wire::AwsJson => match custom_key_store_id {
                None => br#"{"NumberOfBytes":64}"#.to_vec(),
                Some(id) => {
                    format!(r#"{{"NumberOfBytes":64,"CustomKeyStoreId":"{id}"}}"#).into_bytes()
                }
            },
            Wire::RpcV2Cbor => {
                let mut encoder = Encoder::new(Vec::new());
                encoder.begin_map().str("NumberOfBytes").integer(64);
                if let Some(id) = custom_key_store_id {
                    encoder.str("CustomKeyStoreId").str(id);
                }
                encoder.end();
                encoder.into_writer()
            }
        }
    }

    fn generate_random_output(self, plaintext: &[u8]) -> Vec<u8> {
        match self {
            Wire::AwsJson => {
                format!(r#"{{"Plaintext":"{}"}}"#, base64::encode(plaintext)).into_bytes()
            }
            Wire::RpcV2Cbor => {
                let mut encoder = Encoder::new(Vec::new());
                encoder
                    .begin_map()
                    .str("Plaintext")
                    .blob_bytes(plaintext)
                    .end();
                encoder.into_writer()
            }
        }
    }

    /// An error envelope carrying `error_code` where the protocol's parser reads it.
    ///
    /// `json_errors` accepts `code` and `__type`. `cbor_errors` accepts `__type` alone, so
    /// `__type` serves both.
    fn error_output(self, error_code: &str) -> Vec<u8> {
        match self {
            Wire::AwsJson => format!(r#"{{"__type":"{error_code}"}}"#).into_bytes(),
            Wire::RpcV2Cbor => {
                let mut encoder = Encoder::new(Vec::new());
                encoder.begin_map().str("__type").str(error_code).end();
                encoder.into_writer()
            }
        }
    }
}

/// Send one throwaway request and read the protocol off the content type the client set.
///
/// An unrecognized content type panics here, so a protocol change stays visible.
async fn detect_wire() -> Wire {
    let (http_client, rx) = capture_request(None);
    let conf = Config::builder()
        .http_client(http_client)
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .build();
    let _ = kms::Client::from_conf(conf)
        .generate_random()
        .number_of_bytes(64)
        .send()
        .await;
    match rx.expect_request().headers().get("content-type") {
        Some("application/x-amz-json-1.1") => Wire::AwsJson,
        Some("application/cbor") => Wire::RpcV2Cbor,
        other => panic!("unrecognized KMS wire protocol: {other:?}"),
    }
}

#[tokio::test]
async fn generate_random_cn() {
    let wire = detect_wire().await;
    let plaintext = base64::decode(PLAINTEXT_B64).unwrap();
    let http_client = StaticReplayClient::new(vec![ReplayEvent::new(
        http_1x::Request::builder()
            .uri(
                format!("https://kms.cn-north-1.amazonaws.com.cn{}", wire.path())
                    .parse::<Uri>()
                    .unwrap(),
            )
            .body(SdkBody::from(wire.generate_random_input(None)))
            .unwrap(),
        http_1x::Response::builder()
            .status(http_1x::StatusCode::from_u16(200).unwrap())
            .body(SdkBody::from(wire.generate_random_output(&plaintext)))
            .unwrap(),
    )]);
    let conf = Config::builder()
        .http_client(http_client.clone())
        .region(Region::new("cn-north-1"))
        .credentials_provider(Credentials::for_tests())
        .build();
    let client = kms::Client::from_conf(conf);
    let _ = client
        .generate_random()
        .number_of_bytes(64)
        .send()
        .await
        .expect("success");

    assert_eq!(http_client.actual_requests().count(), 1);
    http_client.assert_requests_match(&[]);
}

#[cfg(feature = "test-util")]
#[tokio::test]
async fn generate_random() {
    let wire = detect_wire().await;
    let plaintext = base64::decode(PLAINTEXT_B64).unwrap();
    let request_body = wire.generate_random_input(None);

    // `relaxed_requests_match` ignores `authorization`, `user-agent`, and `x-amz-user-agent`.
    // The expected request below declares the headers that do get compared.
    let mut expected = http_1x::Request::builder()
        .header("content-type", wire.content_type())
        .header("content-length", request_body.len().to_string())
        .header("x-amz-date", "20090213T233130Z")
        .uri(
            format!("https://kms.us-east-1.amazonaws.com{}", wire.path())
                .parse::<Uri>()
                .unwrap(),
        );
    if wire == Wire::AwsJson {
        expected = expected.header("x-amz-target", "TrentService.GenerateRandom");
    }

    let http_client = StaticReplayClient::new(vec![ReplayEvent::new(
        expected.body(SdkBody::from(request_body)).unwrap(),
        http_1x::Response::builder()
            .status(http_1x::StatusCode::from_u16(200).unwrap())
            .body(SdkBody::from(wire.generate_random_output(&plaintext)))
            .unwrap(),
    )]);
    let conf = Config::builder()
        .http_client(http_client.clone())
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests_with_session_token())
        .with_test_defaults()
        .build();
    let client = kms::Client::from_conf(conf);
    let resp = client
        .generate_random()
        .number_of_bytes(64)
        .send()
        .await
        .expect("request should succeed");
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
    http_client.relaxed_requests_match();
}

#[tokio::test]
async fn generate_random_malformed_response() {
    let wire = detect_wire().await;
    let plaintext = base64::decode(PLAINTEXT_B64).unwrap();
    let mut response_body = wire.generate_random_output(&plaintext);
    // Drop the terminator so the parser runs off the end of the body. That is the closing `}`
    // under JSON and the `break` byte of the indefinite-length map under CBOR.
    response_body.pop();

    let http_client = StaticReplayClient::new(vec![ReplayEvent::new(
        http_1x::Request::builder()
            .body(SdkBody::from(wire.generate_random_input(None)))
            .unwrap(),
        http_1x::Response::builder()
            .status(http_1x::StatusCode::from_u16(200).unwrap())
            .body(SdkBody::from(response_body))
            .unwrap(),
    )]);
    let conf = Config::builder()
        .http_client(http_client.clone())
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .build();
    let client = kms::Client::from_conf(conf);
    client
        .generate_random()
        .number_of_bytes(64)
        .send()
        .await
        .expect_err("response was malformed");
}

#[cfg(feature = "test-util")]
#[tokio::test]
async fn generate_random_keystore_not_found() {
    let wire = detect_wire().await;
    let request_body = wire.generate_random_input(Some("does not exist"));
    let response_body = wire.error_output("CustomKeyStoreNotFoundException");

    let mut expected = http_1x::Request::builder()
        .header("content-type", wire.content_type())
        .header("content-length", request_body.len().to_string())
        .header("x-amz-date", "20090213T233130Z")
        .uri(
            format!("https://kms.us-east-1.amazonaws.com{}", wire.path())
                .parse::<Uri>()
                .unwrap(),
        );
    if wire == Wire::AwsJson {
        expected = expected.header("x-amz-target", "TrentService.GenerateRandom");
    }

    let http_client = StaticReplayClient::new(vec![ReplayEvent::new(
        expected.body(SdkBody::from(request_body)).unwrap(),
        http_1x::Response::builder()
            .status(http_1x::StatusCode::from_u16(400).unwrap())
            .header("x-amzn-requestid", "bfe81a0a-9a08-4e71-9910-cdb5ab6ea3b6")
            .header(
                "cache-control",
                "no-cache, no-store, must-revalidate, private",
            )
            .header("expires", "0")
            .header("pragma", "no-cache")
            .header("date", "Fri, 05 Mar 2021 15:01:40 GMT")
            .header("content-type", wire.content_type())
            .header("content-length", response_body.len().to_string())
            .body(SdkBody::from(response_body))
            .unwrap(),
    )]);
    let conf = Config::builder()
        .http_client(http_client.clone())
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests_with_session_token())
        .with_test_defaults()
        .build();
    let client = kms::Client::from_conf(conf);

    let err = client
        .generate_random()
        .number_of_bytes(64)
        .custom_key_store_id("does not exist")
        .send()
        .await
        .expect_err("key store doesn't exist");

    let inner = match err {
        SdkError::ServiceError(context) => context.into_err(),
        other => panic!("Incorrect error received: {:}", other),
    };
    assert!(inner.is_custom_key_store_not_found_exception());
    assert_eq!(
        inner.request_id(),
        Some("bfe81a0a-9a08-4e71-9910-cdb5ab6ea3b6")
    );
    http_client.relaxed_requests_match();
}

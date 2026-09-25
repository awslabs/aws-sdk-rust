/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg(feature = "test-util")]

//! S3 echoes customer-supplied strings back in response headers, and an HTTP header value may
//! contain any octet except a control character — so a value can arrive that is not valid UTF-8 and
//! therefore cannot be a Rust `String`.
//!
//! `GetObject` is a useful case to pin because its output streams, which means it is deserialized by
//! `deserialize_streaming_with_config` rather than the buffered path.

use aws_sdk_s3::{config::Region, Client, Config};
use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::{
    AfterDeserializationInterceptorContextRef, BeforeSerializationInterceptorContextRef,
};
use aws_smithy_runtime_api::client::interceptors::Intercept;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_runtime_api::http::NonUtf8HeaderHandling;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::ConfigBag;
use std::sync::{Arc, Mutex};

/// A synthetic `x-amz-expiration` whose rule id is not valid UTF-8. A lone `0xE9` is a valid header
/// octet (obs-text per RFC 7230) but not a valid UTF-8 sequence.
const NON_UTF8_EXPIRATION: &[u8] =
    b"expiry-date=\"Fri, 21 Dec 2012 00:00:00 GMT\", rule-id=\"rule-\xe9\"";

/// Opts into [`NonUtf8HeaderHandling::Skip`] and records which header values were still unreadable
/// once deserialization finished. `Skip` leaves the header in place, so this is how a caller recovers
/// the octets.
#[derive(Clone, Debug, Default)]
struct SkipNonUtf8Headers {
    seen: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
}

impl SkipNonUtf8Headers {
    fn seen(&self) -> Vec<(String, Vec<u8>)> {
        self.seen.lock().unwrap().clone()
    }
}

impl Intercept for SkipNonUtf8Headers {
    fn name(&self) -> &'static str {
        "SkipNonUtf8Headers"
    }

    fn read_before_execution(
        &self,
        _context: &BeforeSerializationInterceptorContextRef<'_>,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        cfg.interceptor_state()
            .store_put(NonUtf8HeaderHandling::Skip);
        Ok(())
    }

    fn read_after_deserialization(
        &self,
        context: &AfterDeserializationInterceptorContextRef<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        // Overwrite rather than append, so this describes the response that was deserialized even
        // after a retry.
        *self.seen.lock().unwrap() = context
            .response()
            .headers()
            .iter_bytes()
            .filter(|(_, value)| std::str::from_utf8(value).is_err())
            .map(|(name, value)| (name.to_owned(), value.to_vec()))
            .collect();
        Ok(())
    }
}

fn request() -> http_1x::Request<SdkBody> {
    http_1x::Request::builder()
        .uri("https://some-bucket.s3.us-east-1.amazonaws.com/some-key")
        .body(SdkBody::empty())
        .unwrap()
}

/// Note this conversion admits the non-UTF-8 value at all: `ReplayEvent::new` converts into the
/// SDK's own response type, which previously rejected the whole header map.
fn response(status: u16, expiration: &[u8], body: &'static str) -> http_1x::Response<SdkBody> {
    http_1x::Response::builder()
        .status(status)
        .header(
            "x-amz-expiration",
            http_1x::HeaderValue::from_bytes(expiration).unwrap(),
        )
        .body(SdkBody::from(body))
        .unwrap()
}

fn replay_client() -> StaticReplayClient {
    StaticReplayClient::new(vec![ReplayEvent::new(
        request(),
        response(200, NON_UTF8_EXPIRATION, "some-object-contents"),
    )])
}

fn client(http_client: StaticReplayClient, interceptor: Option<SkipNonUtf8Headers>) -> Client {
    let mut config = Config::builder()
        .region(Region::new("us-east-1"))
        .http_client(http_client)
        .with_test_defaults();
    if let Some(interceptor) = interceptor {
        config = config.interceptor(interceptor);
    }
    Client::from_conf(config.build())
}

#[tokio::test]
async fn non_utf8_expiration_header_fails_by_default() {
    let client = client(replay_client(), None);

    let err = client
        .get_object()
        .bucket("some-bucket")
        .key("some-key")
        .send()
        .await
        .expect_err("a value the service sent must not be silently discarded");

    // The failure names the member that could not be parsed, rather than surfacing as a dispatch
    // failure for the whole response.
    let msg = format!(
        "{}",
        aws_smithy_types::error::display::DisplayErrorContext(&err)
    );
    assert!(msg.contains("expiration"), "{msg}");
}

#[tokio::test]
async fn skip_yields_no_expiration_and_leaves_the_octets_readable() {
    let interceptor = SkipNonUtf8Headers::default();
    let client = client(replay_client(), Some(interceptor.clone()));

    let out = client
        .get_object()
        .bucket("some-bucket")
        .key("some-key")
        .send()
        .await
        .expect("Skip tolerates the unreadable value");

    // The member is absent rather than holding a re-encoded approximation of the rule id.
    assert_eq!(None, out.expiration());

    // ...and the header was never removed, so the octets are still there byte for byte.
    assert_eq!(
        vec![("x-amz-expiration".to_string(), NON_UTF8_EXPIRATION.to_vec())],
        interceptor.seen(),
    );
}

/// `read_after_deserialization` runs once per *attempt*, including an attempt that ends in a
/// deserialized service error. An interceptor that captures header octets therefore has to overwrite
/// rather than append, or it accumulates a value per attempt with no way to tell which response each
/// one came from.
///
/// This pins that the captured octets belong to the attempt that actually produced the output.
#[tokio::test]
async fn only_the_final_attempt_is_captured() {
    // The unreadable value differs per attempt so they cannot be confused. The retried response is
    // an error, whose shape binds nothing to `x-amz-expiration` — that is fine, and is itself the
    // point that a header bound to no member is now harmless.
    const FIRST_ATTEMPT_EXPIRATION: &[u8] = b"rule-id=\"first-\xe9\"";

    let http_client = StaticReplayClient::new(vec![
        ReplayEvent::new(
            request(),
            response(
                503,
                FIRST_ATTEMPT_EXPIRATION,
                "<Error><Code>SlowDown</Code></Error>",
            ),
        ),
        ReplayEvent::new(
            request(),
            response(200, NON_UTF8_EXPIRATION, "some-object-contents"),
        ),
    ]);

    let interceptor = SkipNonUtf8Headers::default();
    let config = Config::builder()
        .region(Region::new("us-east-1"))
        .http_client(http_client)
        .retry_config(aws_sdk_s3::config::retry::RetryConfig::standard())
        .sleep_impl(aws_smithy_async::test_util::InstantSleep::unlogged())
        .interceptor(interceptor.clone())
        .with_test_defaults()
        .build();

    let out = Client::from_conf(config)
        .get_object()
        .bucket("some-bucket")
        .key("some-key")
        .send()
        .await
        .expect("the retry succeeds");

    assert_eq!(None, out.expiration());

    // Exactly one entry, and it is the successful attempt's — not the 503's, and not both.
    assert_eq!(
        vec![("x-amz-expiration".to_string(), NON_UTF8_EXPIRATION.to_vec())],
        interceptor.seen(),
    );
}

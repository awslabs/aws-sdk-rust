/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg(feature = "wiremock")]

mod test_operation;

use aws_smithy_async::rt::sleep::{SharedAsyncSleep, TokioSleep};
use aws_smithy_client::test_connection::wire_mock;
use aws_smithy_client::test_connection::wire_mock::{check_matches, RecordedEvent, ReplayedEvent};
use aws_smithy_client::{hyper_ext, Builder};
use aws_smithy_client::{match_events, Client};
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::operation;
use aws_smithy_http::operation::Operation;
use aws_smithy_types::retry::ReconnectMode;
use aws_smithy_types::timeout::{OperationTimeoutConfig, TimeoutConfig};
use http::Uri;
use http_body::combinators::BoxBody;
use hyper::client::{Builder as HyperBuilder, HttpConnector};
use std::convert::Infallible;
use std::time::Duration;
use test_operation::{TestOperationParser, TestRetryClassifier};
use tower::layer::util::Identity;
use wire_mock::ev;

fn end_of_test() -> &'static str {
    "end_of_test"
}

fn test_operation(
    uri: Uri,
    retryable: bool,
) -> Operation<TestOperationParser, TestRetryClassifier> {
    let mut req = operation::Request::new(
        http::Request::builder()
            .uri(uri)
            .body(SdkBody::from("request body"))
            .unwrap(),
    );
    if !retryable {
        req = req
            .augment(|req, _conf| {
                Ok::<_, Infallible>(
                    req.map(|_| SdkBody::from_dyn(BoxBody::new(SdkBody::from("body")))),
                )
            })
            .unwrap();
    }
    Operation::new(req, TestOperationParser).with_retry_classifier(TestRetryClassifier)
}

async fn h1_and_h2(events: Vec<ReplayedEvent>, match_clause: impl Fn(&[RecordedEvent])) {
    wire_level_test(events.clone(), |_b| {}, |b| b, &match_clause).await;
    wire_level_test(
        events,
        |b| {
            b.http2_only(true);
        },
        |b| b,
        match_clause,
    )
    .await;
    println!("h2 ok!");
}

/// Repeatedly send test operation until `end_of_test` is received
///
/// When the test is over, match_clause is evaluated
async fn wire_level_test(
    events: Vec<ReplayedEvent>,
    hyper_builder_settings: impl Fn(&mut HyperBuilder),
    client_builder_settings: impl Fn(Builder) -> Builder,
    match_clause: impl Fn(&[RecordedEvent]),
) {
    let connection = wire_mock::WireLevelTestConnection::spinup(events).await;

    let http_connector = HttpConnector::new_with_resolver(connection.dns_resolver());
    let mut hyper_builder = hyper::Client::builder();
    hyper_builder_settings(&mut hyper_builder);
    let hyper_adapter = hyper_ext::Adapter::builder()
        .hyper_builder(hyper_builder)
        .build(http_connector);
    let client = client_builder_settings(
        Client::builder().reconnect_mode(ReconnectMode::ReconnectOnTransientError),
    )
    .connector(hyper_adapter)
    .middleware(Identity::new())
    .operation_timeout_config(OperationTimeoutConfig::from(
        &TimeoutConfig::builder()
            .operation_attempt_timeout(Duration::from_millis(100))
            .build(),
    ))
    .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
    .build();
    loop {
        match client
            .call(test_operation(
                connection.endpoint_url().parse().unwrap(),
                false,
            ))
            .await
        {
            Ok(resp) => {
                tracing::info!("response: {:?}", resp);
                if resp == end_of_test() {
                    break;
                }
            }
            Err(e) => tracing::info!("error: {:?}", e),
        }
    }
    let events = connection.events();
    match_clause(&events);
}

#[tokio::test]
async fn non_transient_errors_no_reconect() {
    h1_and_h2(
        vec![
            ReplayedEvent::status(400),
            ReplayedEvent::with_body(end_of_test()),
        ],
        match_events!(ev!(dns), ev!(connect), ev!(http(400)), ev!(http(200))),
    )
    .await
}

#[tokio::test]
async fn reestablish_dns_on_503() {
    h1_and_h2(
        vec![
            ReplayedEvent::status(503),
            ReplayedEvent::status(503),
            ReplayedEvent::status(503),
            ReplayedEvent::with_body(end_of_test()),
        ],
        match_events!(
            // first request
            ev!(dns),
            ev!(connect),
            ev!(http(503)),
            // second request
            ev!(dns),
            ev!(connect),
            ev!(http(503)),
            // third request
            ev!(dns),
            ev!(connect),
            ev!(http(503)),
            // all good
            ev!(dns),
            ev!(connect),
            ev!(http(200))
        ),
    )
    .await;
}

#[tokio::test]
async fn connection_shared_on_success() {
    h1_and_h2(
        vec![
            ReplayedEvent::ok(),
            ReplayedEvent::ok(),
            ReplayedEvent::status(503),
            ReplayedEvent::with_body(end_of_test()),
        ],
        match_events!(
            ev!(dns),
            ev!(connect),
            ev!(http(200)),
            ev!(http(200)),
            ev!(http(503)),
            ev!(dns),
            ev!(connect),
            ev!(http(200))
        ),
    )
    .await;
}

#[tokio::test]
async fn no_reconnect_when_disabled() {
    use wire_mock::ev;
    wire_level_test(
        vec![
            ReplayedEvent::status(503),
            ReplayedEvent::with_body(end_of_test()),
        ],
        |_b| {},
        |b| b.reconnect_mode(ReconnectMode::ReuseAllConnections),
        match_events!(ev!(dns), ev!(connect), ev!(http(503)), ev!(http(200))),
    )
    .await;
}

#[tokio::test]
async fn connection_reestablished_after_timeout() {
    use wire_mock::ev;
    h1_and_h2(
        vec![
            ReplayedEvent::ok(),
            ReplayedEvent::Timeout,
            ReplayedEvent::ok(),
            ReplayedEvent::Timeout,
            ReplayedEvent::with_body(end_of_test()),
        ],
        match_events!(
            // first connection
            ev!(dns),
            ev!(connect),
            ev!(http(200)),
            // reuse but got a timeout
            ev!(timeout),
            // so we reconnect
            ev!(dns),
            ev!(connect),
            ev!(http(200)),
            ev!(timeout),
            ev!(dns),
            ev!(connect),
            ev!(http(200))
        ),
    )
    .await;
}

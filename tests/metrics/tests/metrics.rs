/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_config::retry::RetryConfig;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_s3::config::{Credentials, Region, SharedCredentialsProvider};
use aws_smithy_observability::global::set_telemetry_provider;
use aws_smithy_observability::TelemetryProvider;
use aws_smithy_observability_otel::meter::OtelMeterProvider;
use aws_smithy_runtime::client::http::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::body::SdkBody;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::data::{Histogram, ResourceMetrics};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::testing::metrics::InMemoryMetricsExporter;
use serial_test::serial;
use std::borrow::Cow;
use std::sync::Arc;

// Note, all of these tests are written with a multi threaded runtime since OTel requires that to work
// and they are all run serially since they touch global state
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial]
async fn service_clients_get_unique_scope_names() {
    let (meter_provider, exporter) = init_metrics();
    let config = make_config(false);
    make_s3_call(&config).await;
    make_ddb_call(&config).await;

    meter_provider.flush().unwrap();
    let finished_metrics = exporter.get_finished_metrics().unwrap();

    let scope_names: &Vec<Cow<'_, str>> = &finished_metrics[0]
        .scope_metrics
        .iter()
        .map(|scope_metric| scope_metric.scope.clone().name)
        .collect();

    // Metrics aren't necessarily aggregated in the order they were first emitted
    assert!(scope_names.contains(&Cow::from("aws-sdk-s3")));
    assert!(scope_names.contains(&Cow::from("aws-sdk-dynamodb")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial]
async fn correct_metrics_collected() {
    let (meter_provider, exporter) = init_metrics();
    make_s3_call(&make_config(false)).await;

    meter_provider.flush().unwrap();
    let finished_metrics = exporter.get_finished_metrics().unwrap();

    let extracted_metric_names: &Vec<Cow<'_, str>> = &finished_metrics[0].scope_metrics[0]
        .metrics
        .iter()
        .map(|metric| metric.name.clone())
        .collect();

    // Correct metric names emitted
    assert!(extracted_metric_names.contains(&Cow::from("smithy.client.call.duration")));
    assert!(extracted_metric_names.contains(&Cow::from("smithy.client.call.attempt.duration")));

    let call_duration =
        extract_metric_data::<Histogram<f64>>(&finished_metrics, "smithy.client.call.duration")
            .data_points[0]
            .sum;

    let attempt_duration = extract_metric_data::<Histogram<f64>>(
        &finished_metrics,
        "smithy.client.call.attempt.duration",
    )
    .data_points[0]
        .sum;

    // Both metrics have a non-zero value
    assert!(call_duration > 0.0);
    assert!(attempt_duration > 0.0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[serial]
async fn metrics_have_expected_attributes() {
    let (meter_provider, exporter) = init_metrics();
    make_s3_call(&make_config(true)).await;

    meter_provider.flush().unwrap();
    let finished_metrics = exporter.get_finished_metrics().unwrap();

    // Both metrics contain the method and service attributes
    let call_duration_attributes =
        extract_metric_attributes(&finished_metrics, "smithy.client.call.duration");
    assert!(call_duration_attributes[0].contains(&KeyValue::new("rpc.method", "GetObject")));
    assert!(call_duration_attributes[0].contains(&KeyValue::new("rpc.service", "s3")));

    let attempt_duration_attributes =
        extract_metric_attributes(&finished_metrics, "smithy.client.call.attempt.duration");
    assert!(attempt_duration_attributes[0].contains(&KeyValue::new("rpc.method", "GetObject")));
    assert!(attempt_duration_attributes[0].contains(&KeyValue::new("rpc.service", "s3")));

    // The attempt metric contains an attempt counter attribute that correctly increments
    assert!(attempt_duration_attributes
        .iter()
        .find(|attrs| attrs.contains(&KeyValue::new("attempt", 1)))
        .is_some());
    assert!(attempt_duration_attributes
        .iter()
        .find(|attrs| attrs.contains(&KeyValue::new("attempt", 2)))
        .is_some());
}

// Util functions
fn init_metrics() -> (Arc<OtelMeterProvider>, InMemoryMetricsExporter) {
    let exporter = InMemoryMetricsExporter::default();
    let reader = PeriodicReader::builder(exporter.clone(), Tokio).build();
    let otel_mp = SdkMeterProvider::builder().with_reader(reader).build();

    let sdk_mp = Arc::new(OtelMeterProvider::new(otel_mp));
    let sdk_ref = sdk_mp.clone();
    let sdk_tp = TelemetryProvider::builder().meter_provider(sdk_mp).build();

    let _ = set_telemetry_provider(sdk_tp);

    (sdk_ref, exporter)
}

fn new_replay_client(num_requests: usize, with_retry: bool) -> StaticReplayClient {
    let mut events = Vec::with_capacity(num_requests);
    let mut start = 0;

    if with_retry {
        events.push(ReplayEvent::new(
            http::Request::builder().body(SdkBody::empty()).unwrap(),
            http::Response::builder()
                .status(500)
                .body(SdkBody::empty())
                .unwrap(),
        ));
        start += 1;
    }

    for _ in start..num_requests {
        events.push(ReplayEvent::new(
            http::Request::builder().body(SdkBody::empty()).unwrap(),
            http::Response::builder().body(SdkBody::empty()).unwrap(),
        ))
    }
    StaticReplayClient::new(events)
}

fn extract_metric_data<'a, T: 'static>(
    metrics: &'a Vec<ResourceMetrics>,
    metric_name: &str,
) -> &'a T {
    &metrics[0].scope_metrics[0]
        .metrics
        .iter()
        .find(|metric| metric.name == metric_name)
        .unwrap()
        .data
        .as_any()
        .downcast_ref::<T>()
        .unwrap()
}

fn extract_metric_attributes<'a>(
    metrics: &'a Vec<ResourceMetrics>,
    metric_name: &str,
) -> Vec<Vec<KeyValue>> {
    extract_metric_data::<Histogram<f64>>(metrics, metric_name)
        .data_points
        .iter()
        .map(|dp| dp.attributes.clone())
        .collect()
}

async fn make_s3_call(config: &SdkConfig) {
    let s3_client = aws_sdk_s3::Client::new(config);
    let _ = s3_client
        .get_object()
        .bucket("some-test-bucket")
        .key("test.txt")
        .send()
        .await;
}

async fn make_ddb_call(config: &SdkConfig) {
    let ddb_client = aws_sdk_dynamodb::Client::new(&config);
    let _ = ddb_client
        .get_item()
        .table_name("test-table")
        .key("foo", AttributeValue::Bool(true))
        .send()
        .await;
}

fn make_config(with_retry: bool) -> SdkConfig {
    SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .region(Region::new("us-east-1"))
        .http_client(new_replay_client(2, with_retry))
        .retry_config(RetryConfig::standard())
        .build()
}

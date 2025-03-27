/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::data::Histogram;
use serial_test::serial;
use std::borrow::Cow;
use utils::{
    extract_metric_attributes, extract_metric_data, init_metrics, make_config, make_ddb_call,
    make_s3_call,
};
mod utils;

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

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::{sync::Arc, time::Instant};

use aws_config::{retry::RetryConfig, Region, SdkConfig};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_s3::config::{Credentials, SharedCredentialsProvider};
use aws_smithy_observability::{global::set_telemetry_provider, TelemetryProvider};
use aws_smithy_observability_otel::meter::OtelMeterProvider;
use aws_smithy_runtime::client::http::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::body::SdkBody;
use opentelemetry::KeyValue;
use opentelemetry_sdk::{
    metrics::{
        data::{Histogram, ResourceMetrics},
        PeriodicReader, SdkMeterProvider,
    },
    runtime::Tokio,
    testing::metrics::InMemoryMetricsExporter,
};
use tracing::{
    field::{Field, Visit},
    span::{Attributes, Id},
    Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

#[allow(unused)]
pub(crate) fn init_metrics() -> (Arc<OtelMeterProvider>, InMemoryMetricsExporter) {
    let exporter = InMemoryMetricsExporter::default();
    let reader = PeriodicReader::builder(exporter.clone(), Tokio).build();
    let otel_mp = SdkMeterProvider::builder().with_reader(reader).build();

    let sdk_mp = Arc::new(OtelMeterProvider::new(otel_mp));
    let sdk_ref = sdk_mp.clone();
    let sdk_tp = TelemetryProvider::builder().meter_provider(sdk_mp).build();

    let _ = set_telemetry_provider(sdk_tp);

    (sdk_ref, exporter)
}

pub(crate) fn new_replay_client(num_requests: usize, with_retry: bool) -> StaticReplayClient {
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

#[allow(unused)]
pub(crate) fn extract_metric_data<'a, T: 'static>(
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

#[allow(unused)]
pub(crate) fn extract_metric_attributes<'a>(
    metrics: &'a Vec<ResourceMetrics>,
    metric_name: &str,
) -> Vec<Vec<KeyValue>> {
    extract_metric_data::<Histogram<f64>>(metrics, metric_name)
        .data_points
        .iter()
        .map(|dp| dp.attributes.clone())
        .collect()
}

pub(crate) async fn make_s3_call(config: &SdkConfig) {
    let s3_client = aws_sdk_s3::Client::new(config);
    let _ = s3_client
        .get_object()
        .bucket("some-test-bucket")
        .key("test.txt")
        .send()
        .await;
}

pub(crate) async fn make_ddb_call(config: &SdkConfig) {
    let ddb_client = aws_sdk_dynamodb::Client::new(&config);
    let _ = ddb_client
        .get_item()
        .table_name("test-table")
        .key("foo", AttributeValue::Bool(true))
        .send()
        .await;
}

pub(crate) fn make_config(with_retry: bool) -> SdkConfig {
    SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .region(Region::new("us-east-1"))
        .http_client(new_replay_client(2, with_retry))
        .retry_config(RetryConfig::standard())
        .build()
}

/// Util for printing spans for debugging purposes. Can be used with:
/// ```
/// let subscriber = tracing_subscriber::registry::Registry::default().with(PrintLayer);
/// let _guard = tracing::subscriber::set_default(subscriber);
/// ```
///
/// Outputs logs like:
/// ```sh
/// Span Created: s3.GetObject
/// ATTR: rpc.service: s3
/// ATTR: rpc.method: GetObject
/// ATTR: sdk_invocation_id: 7048479
/// Span Created: invoke
/// ATTR: rpc.service: s3
/// ATTR: rpc.method: GetObject
/// Span Created: apply_configuration
/// Span Closed: apply_configuration, Duration:  150
/// ```
#[allow(unused)]
pub(crate) struct PrintLayer;

#[allow(unused)]
pub(crate) struct Timing {
    started_at: Instant,
}

impl<S> Layer<S> for PrintLayer
where
    S: Subscriber,
    S: for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();

        println!("Span Created: {}", span.metadata().name());
        span.extensions_mut().insert(Timing {
            started_at: Instant::now(),
        });

        attrs.values().record(&mut PrintVisitor);
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).unwrap();

        let started_at = span.extensions().get::<Timing>().unwrap().started_at;

        println!(
            "Span Closed: {}, Duration:  {}",
            span.metadata().name(),
            (Instant::now() - started_at).as_micros(),
        );
    }
}

#[allow(unused)]
pub(crate) struct PrintVisitor;

impl Visit for PrintVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let field_name = field.name();
        let field_value = format!("{value:?}").replace("\"", "");
        println!("ATTR: {field_name}: {field_value}")
    }
}

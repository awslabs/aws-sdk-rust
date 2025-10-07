/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/* Automatically managed default lints */
#![cfg_attr(docsrs, feature(doc_cfg))]
/* End of automatically managed default lints */
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    unreachable_pub,
    rust_2018_idioms
)]
// The `opentelemetry_sdk` crate uses std::sync::atomic::{AtomicI64, AtomicU64} which are not available on powerpc
#![cfg(not(target_arch = "powerpc"))]

//! Smithy Observability OpenTelemetry
//TODO(smithyobservability): once we have finalized everything and integrated metrics with our runtime
// libraries update this with detailed usage docs and examples

pub mod attributes;
pub mod meter;

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use crate::meter::OtelMeterProvider;
    use aws_smithy_observability::{
        global::{get_telemetry_provider, set_telemetry_provider},
        TelemetryProvider,
    };
    use opentelemetry_sdk::metrics::{data::Sum, PeriodicReader, SdkMeterProvider};
    use opentelemetry_sdk::runtime::Tokio;
    use opentelemetry_sdk::testing::metrics::InMemoryMetricsExporter;

    // Without these tokio settings this test just stalls forever on flushing the metrics pipeline
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn can_construct_set_and_use_otel_as_global_telemetry_provider() {
        // Create the OTel metrics objects
        let exporter = InMemoryMetricsExporter::default();
        let reader = PeriodicReader::builder(exporter.clone(), Tokio).build();
        let otel_mp = SdkMeterProvider::builder().with_reader(reader).build();

        // Create the SDK metrics types from the OTel objects
        let sdk_mp = Arc::new(OtelMeterProvider::new(otel_mp));
        let sdk_ref = sdk_mp.clone();
        let sdk_tp = TelemetryProvider::builder().meter_provider(sdk_mp).build();

        // Set the global TelemetryProvider and then get it back out
        let _ = set_telemetry_provider(sdk_tp);
        let global_tp = get_telemetry_provider().unwrap();

        // Create an instrument and record a value
        let global_meter = global_tp
            .meter_provider()
            .get_meter("TestGlobalMeter", None);

        let mono_counter = global_meter
            .create_monotonic_counter("TestMonoCounter")
            .build();
        mono_counter.add(4, None, None);

        sdk_ref.flush().unwrap();
        let finished_metrics = exporter.get_finished_metrics().unwrap();

        let extracted_mono_counter_data = &finished_metrics[0].scope_metrics[0].metrics[0]
            .data
            .as_any()
            .downcast_ref::<Sum<u64>>()
            .unwrap()
            .data_points[0]
            .value;
        assert_eq!(extracted_mono_counter_data, &4);
    }
}

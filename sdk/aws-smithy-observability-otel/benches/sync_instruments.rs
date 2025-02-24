/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_observability::meter::Meter;
use aws_smithy_observability::TelemetryProvider;
use aws_smithy_observability_otel::meter::OtelMeterProvider;
use criterion::{criterion_group, criterion_main, Criterion};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::testing::metrics::InMemoryMetricsExporter;

use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;
use std::sync::Arc;

async fn record_sync_instruments(dyn_sdk_meter: Meter) {
    //Create all 3 sync instruments and record some data for each
    let mono_counter = dyn_sdk_meter
        .create_monotonic_counter("TestMonoCounter")
        .build();
    mono_counter.add(4, None, None);
    let ud_counter = dyn_sdk_meter
        .create_up_down_counter("TestUpDownCounter")
        .build();
    ud_counter.add(-6, None, None);
    let histogram = dyn_sdk_meter
        .create_histogram("TestHistogram".to_string())
        .build();
    histogram.record(1.234, None, None);
}

fn sync_instruments_benchmark(c: &mut Criterion) {
    #[global_allocator]
    static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;
    let reg = Region::new(&GLOBAL);

    // Setup the Otel MeterProvider (which needs to be done inside an async runtime)
    // The runtime is reused later for running the bench function
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let otel_mp = runtime.block_on(async {
        let exporter = InMemoryMetricsExporter::default();
        let reader = PeriodicReader::builder(exporter.clone(), Tokio).build();
        SdkMeterProvider::builder().with_reader(reader).build()
    });
    // Create the SDK metrics types from the OTel objects
    let sdk_mp = Arc::new(OtelMeterProvider::new(otel_mp));
    let sdk_tp = TelemetryProvider::builder().meter_provider(sdk_mp).build();

    // Get the dyn versions of the SDK metrics objects
    let dyn_sdk_mp = sdk_tp.meter_provider();
    let dyn_sdk_meter = dyn_sdk_mp.get_meter("TestMeter", None);

    c.bench_function("sync_instruments", |b| {
        b.to_async(&runtime)
            .iter(|| async { record_sync_instruments(dyn_sdk_meter.clone()) });
    });

    println!("FINISHING");
    println!("Stats at end: {:#?}", reg.change());
}

criterion_group!(benches, sync_instruments_benchmark);
criterion_main!(benches);

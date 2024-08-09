/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_cbor::decode::Decoder;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn blob_benchmark(c: &mut Criterion) {
    // Indefinite length blob containing bytes corresponding to `indefinite-byte, chunked, on each comma`.
    let blob_indefinite_bytes = [
        0x5f, 0x50, 0x69, 0x6e, 0x64, 0x65, 0x66, 0x69, 0x6e, 0x69, 0x74, 0x65, 0x2d, 0x62, 0x79,
        0x74, 0x65, 0x2c, 0x49, 0x20, 0x63, 0x68, 0x75, 0x6e, 0x6b, 0x65, 0x64, 0x2c, 0x4e, 0x20,
        0x6f, 0x6e, 0x20, 0x65, 0x61, 0x63, 0x68, 0x20, 0x63, 0x6f, 0x6d, 0x6d, 0x61, 0xff,
    ];

    c.bench_function("blob", |b| {
        b.iter(|| {
            let mut decoder = Decoder::new(&blob_indefinite_bytes);
            let _ = black_box(decoder.blob());
        })
    });
}

criterion_group!(benches, blob_benchmark);
criterion_main!(benches);

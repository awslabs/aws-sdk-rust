/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Compares reading a file through `ByteStream::read_from()` against reading it into memory
//! first. See <https://github.com/smithy-lang/smithy-rs/issues/2833>.

use aws_smithy_types::byte_stream::ByteStream;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::io::Write;
use tempfile::NamedTempFile;

const FILE_SIZE: usize = 64 * 1024 * 1024;

fn make_file() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    // Non-zero, non-repeating-ish content so nothing can shortcut the read.
    let chunk: Vec<u8> = (0..1024 * 1024).map(|i| (i % 251) as u8).collect();
    for _ in 0..(FILE_SIZE / chunk.len()) {
        file.write_all(&chunk).unwrap();
    }
    file.flush().unwrap();
    file
}

fn bench(c: &mut Criterion) {
    let file = make_file();
    let path = file.path().to_path_buf();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("bytestream_64mib");
    group.throughput(Throughput::Bytes(FILE_SIZE as u64));
    group.sample_size(20);

    group.bench_function("read_into_memory", |b| {
        b.to_async(&rt).iter(|| async {
            let bytes = tokio::fs::read(&path).await.unwrap();
            let stream = ByteStream::from(bytes);
            let collected = stream.collect().await.unwrap();
            assert_eq!(collected.into_bytes().len(), FILE_SIZE);
        })
    });

    group.bench_function(BenchmarkId::new("from_path", "default"), |b| {
        b.to_async(&rt).iter(|| async {
            let stream = ByteStream::from_path(&path).await.unwrap();
            let collected = stream.collect().await.unwrap();
            assert_eq!(collected.into_bytes().len(), FILE_SIZE);
        })
    });

    for buffer_size in [4 * 1024, 16 * 1024, 64 * 1024, 256 * 1024, 1024 * 1024] {
        group.bench_with_input(
            BenchmarkId::new("from_path", buffer_size),
            &buffer_size,
            |b, &buffer_size| {
                b.to_async(&rt).iter(|| async {
                    let stream = ByteStream::read_from()
                        .path(&path)
                        .buffer_size(buffer_size)
                        .build()
                        .await
                        .unwrap();
                    let collected = stream.collect().await.unwrap();
                    assert_eq!(collected.into_bytes().len(), FILE_SIZE);
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);

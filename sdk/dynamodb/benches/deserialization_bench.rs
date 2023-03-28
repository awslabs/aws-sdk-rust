/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_dynamodb::operation::query::Query;
use aws_smithy_http::response::ParseHttpResponse;
use bytes::Bytes;
use criterion::{criterion_group, criterion_main, Criterion};

fn do_bench() {
    let response = http::Response::builder()
        .header("server", "Server")
        .header("date", "Mon, 08 Mar 2021 15:51:23 GMT")
        .header("content-type", "application/x-amz-json-1.0")
        .header("content-length", "1231")
        .header("connection", "keep-alive")
        .header("x-amzn-requestid", "A5FGSJ9ET4OKB8183S9M47RQQBVV4KQNSO5AEMVJF66Q9ASUAAJG")
        .header("x-amz-crc32", "624725176")
        .status(http::StatusCode::from_u16(200).unwrap())
        .body(Bytes::copy_from_slice(br#"{"Count":2,"Items":[{"year":{"N":"2013"},"info":{"M":{"actors":{"L":[{"S":"Daniel Bruhl"},{"S":"Chris Hemsworth"},{"S":"Olivia Wilde"}]},"plot":{"S":"A re-creation of the merciless 1970s rivalry between Formula One rivals James Hunt and Niki Lauda."},"release_date":{"S":"2013-09-02T00:00:00Z"},"image_url":{"S":"http://ia.media-imdb.com/images/M/MV5BMTQyMDE0MTY0OV5BMl5BanBnXkFtZTcwMjI2OTI0OQ@@._V1_SX400_.jpg"},"genres":{"L":[{"S":"Action"},{"S":"Biography"},{"S":"Drama"},{"S":"Sport"}]},"directors":{"L":[{"S":"Ron Howard"}]},"rating":{"N":"8.3"},"rank":{"N":"2"},"running_time_secs":{"N":"7380"}}},"title":{"S":"Rush"}},{"year":{"N":"2013"},"info":{"M":{"actors":{"L":[{"S":"David Matthewman"},{"S":"Ann Thomas"},{"S":"Jonathan G. Neff"}]},"release_date":{"S":"2013-01-18T00:00:00Z"},"plot":{"S":"A rock band plays their music at high volumes, annoying the neighbors."},"genres":{"L":[{"S":"Comedy"},{"S":"Drama"}]},"image_url":{"S":"http://ia.media-imdb.com/images/N/O9ERWAU7FS797AJ7LU8HN09AMUP908RLlo5JF90EWR7LJKQ7@@._V1_SX400_.jpg"},"directors":{"L":[{"S":"Alice Smith"},{"S":"Bob Jones"}]},"rating":{"N":"6.2"},"rank":{"N":"11"},"running_time_secs":{"N":"5215"}}},"title":{"S":"Turn It Down, Or Else!"}}],"ScannedCount":2}"#))
        .unwrap();

    let parser = Query::new();
    let output = <Query as ParseHttpResponse>::parse_loaded(&parser, &response).unwrap();
    assert_eq!(2, output.count);
}

fn bench_group(c: &mut Criterion) {
    c.bench_function("deserialization_bench", |b| b.iter(do_bench));
}

criterion_group!(benches, bench_group);
criterion_main!(benches);

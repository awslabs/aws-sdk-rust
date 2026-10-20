/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Criterion benchmarks for deserializing restJson1 responses with header bindings.
//!
//! Outputs are shaped like S3 `GetObject`/`HeadObject`: string members bound with
//! `@httpHeader`, mostly lowercase `x-amz-*` names plus a few standard mixed-case
//! ones. The responses also carry headers no member binds, as real responses do.

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use aws_smithy_json::protocol::aws_rest_json_1::AwsRestJsonProtocol;
use aws_smithy_runtime_api::http::{Response, StatusCode};
use aws_smithy_schema::protocol::ClientProtocolInner;
use aws_smithy_schema::{shape_id, Schema, ShapeType};
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::ConfigBag;

/// Modeled header names and the value each response carries.
const HEADERS: [(&str, &str); 20] = [
    (
        "x-amz-expiration",
        "expiry-date=\"Fri, 23 Dec 2026 00:00:00 GMT\"",
    ),
    ("ETag", "\"d41d8cd98f00b204e9800998ecf8427e\""),
    ("Cache-Control", "max-age=3600"),
    ("Content-Disposition", "attachment; filename=\"report.csv\""),
    ("Content-Encoding", "gzip"),
    ("Content-Language", "en-US"),
    ("Content-Type", "text/csv"),
    ("x-amz-version-id", "3HL4kqtJlcpXroDTDmJ.rmSpXd3dIbrHY"),
    ("x-amz-website-redirect-location", "/index.html"),
    ("x-amz-server-side-encryption", "aws:kms"),
    (
        "x-amz-server-side-encryption-aws-kms-key-id",
        "arn:aws:kms:us-east-1:123456789012:key/abcd",
    ),
    ("x-amz-storage-class", "STANDARD_IA"),
    ("x-amz-request-charged", "requester"),
    ("x-amz-replication-status", "COMPLETED"),
    ("x-amz-object-lock-mode", "GOVERNANCE"),
    ("x-amz-object-lock-legal-hold", "OFF"),
    ("x-amz-restore", "ongoing-request=\"false\""),
    ("x-amz-checksum-crc32", "AAAAAA=="),
    (
        "x-amz-checksum-sha256",
        "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=",
    ),
    ("x-amz-tagging-count", "3"),
];

macro_rules! header {
    ($static_name:ident, $index:literal) => {
        static $static_name: Schema<'static> = Schema::new_member(
            shape_id!("bench", "Get", "m"),
            ShapeType::String,
            HEADERS[$index].0,
            $index,
        )
        .with_http_header(HEADERS[$index].0);
    };
}

header!(H00, 0);
header!(H01, 1);
header!(H02, 2);
header!(H03, 3);
header!(H04, 4);
header!(H05, 5);
header!(H06, 6);
header!(H07, 7);
header!(H08, 8);
header!(H09, 9);
header!(H10, 10);
header!(H11, 11);
header!(H12, 12);
header!(H13, 13);
header!(H14, 14);
header!(H15, 15);
header!(H16, 16);
header!(H17, 17);
header!(H18, 18);
header!(H19, 19);

static GET_1: Schema<'static> =
    Schema::new_struct(shape_id!("bench", "Get1"), ShapeType::Structure, &[&H00]);
static GET_5: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "Get5"),
    ShapeType::Structure,
    &[&H00, &H01, &H02, &H03, &H04],
);
static GET_20: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "Get20"),
    ShapeType::Structure,
    &[
        &H00, &H01, &H02, &H03, &H04, &H05, &H06, &H07, &H08, &H09, &H10, &H11, &H12, &H13, &H14,
        &H15, &H16, &H17, &H18, &H19,
    ],
);

/// A response carrying the first `bound` modeled headers and the headers every S3
/// response carries.
fn response(bound: usize) -> Response {
    let mut response = Response::new(StatusCode::try_from(200).unwrap(), SdkBody::empty());
    let headers = response.headers_mut();
    for (name, value) in [
        (
            "x-amz-id-2",
            "Uuag1LuByRx9e6j5Onimru9pO4ZVKnJ2Qz7/C1NPcfTWAtRPfTaOFg==",
        ),
        ("x-amz-request-id", "656c76696e6727732072657175657374"),
        ("date", "Wed, 07 Oct 2026 21:32:00 GMT"),
        ("last-modified", "Tue, 06 Oct 2026 12:00:00 GMT"),
        ("accept-ranges", "bytes"),
        ("content-length", "0"),
        ("server", "AmazonS3"),
    ] {
        headers.insert(name, value);
    }
    for (name, value) in &HEADERS[..bound] {
        headers.insert(*name, *value);
    }
    response
}

/// Deserializes `response` as `schema`, reading every member as a string.
#[inline(never)]
fn deserialize(schema: &'static Schema<'static>, response: &Response) -> usize {
    let protocol = AwsRestJsonProtocol::new();
    let cfg = ConfigBag::base();
    let mut deserializer = protocol
        .deserialize_response(response, schema, &cfg)
        .unwrap();
    let mut fields = 0;
    deserializer
        .read_struct(schema, &mut |member, d| {
            black_box(d.read_string(member)?);
            fields += 1;
            Ok(())
        })
        .unwrap();
    fields
}

/// Every benchmarked case: name, output schema and response.
pub fn cases() -> Vec<(String, &'static Schema<'static>, Response)> {
    [(1, &GET_1), (5, &GET_5), (20, &GET_20)]
        .into_iter()
        .map(|(bound, schema)| (format!("headers_{bound}"), schema, response(bound)))
        .collect()
}

fn bench(c: &mut Criterion) {
    for (name, schema, response) in cases() {
        assert_eq!(deserialize(schema, &response), schema.members().len());
        c.bench_function(&format!("response/restJson/{name}"), |b| {
            b.iter(|| deserialize(black_box(schema), black_box(&response)))
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);

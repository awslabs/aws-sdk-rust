/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Criterion benchmarks for schema-based JSON serialization.
//!
//! Structure sizes follow member counts across AWS models (p50 = 2, p90 = 6,
//! p99 = 20), and schemas are `static`s, as generated code emits them. Each case
//! is serialized with the settings of the protocol it represents: awsJson1_0/1_1
//! ignore `@jsonName`, restJson1 honors it.

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use aws_smithy_json::codec::{JsonCodec, JsonCodecSettings};
use aws_smithy_schema::codec::Codec;
use aws_smithy_schema::serde::{SerdeError, SerializableStruct, ShapeSerializer};
use aws_smithy_schema::{shape_id, Schema, ShapeType};
use aws_smithy_types::DateTime;

macro_rules! member {
    ($static_name:ident, $shape:literal, $ty:ident, $name:literal, $index:literal) => {
        static $static_name: Schema<'static> = Schema::new_member(
            shape_id!("bench", $shape, $name),
            ShapeType::$ty,
            $name,
            $index,
        );
    };
    ($static_name:ident, $shape:literal, $ty:ident, $name:literal, $index:literal, json_name = $json:literal) => {
        static $static_name: Schema<'static> = Schema::new_member(
            shape_id!("bench", $shape, $name),
            ShapeType::$ty,
            $name,
            $index,
        )
        .with_json_name($json);
    };
}

// --- 2 members (p50): Tag ---

member!(TAG_KEY, "Tag", String, "Key", 0);
member!(TAG_VALUE, "Tag", String, "Value", 1);
static TAG: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "Tag"),
    ShapeType::Structure,
    &[&TAG_KEY, &TAG_VALUE],
);

struct Tag;

impl SerializableStruct for Tag {
    fn schema(&self) -> &Schema<'_> {
        &TAG
    }

    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        s.write_string(&TAG_KEY, "Environment")?;
        s.write_string(&TAG_VALUE, "Production")
    }
}

// --- 6 members (p90) ---

member!(P90_DOCUMENT_NAME, "P90", String, "DocumentName", 0);
member!(P90_INSTANCE_ID, "P90", String, "InstanceId", 1);
member!(P90_TIMEOUT_SECONDS, "P90", Integer, "TimeoutSeconds", 2);
member!(P90_COMMENT, "P90", String, "Comment", 3);
member!(P90_MAX_CONCURRENCY, "P90", String, "MaxConcurrency", 4);
member!(P90_WITH_DECRYPTION, "P90", Boolean, "WithDecryption", 5);
static P90: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "P90"),
    ShapeType::Structure,
    &[
        &P90_DOCUMENT_NAME,
        &P90_INSTANCE_ID,
        &P90_TIMEOUT_SECONDS,
        &P90_COMMENT,
        &P90_MAX_CONCURRENCY,
        &P90_WITH_DECRYPTION,
    ],
);

struct P90Value;

impl SerializableStruct for P90Value {
    fn schema(&self) -> &Schema<'_> {
        &P90
    }

    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        s.write_string(&P90_DOCUMENT_NAME, "AWS-RunShellScript")?;
        s.write_string(&P90_INSTANCE_ID, "i-0123456789abcdef0")?;
        s.write_integer(&P90_TIMEOUT_SECONDS, 600)?;
        s.write_string(&P90_COMMENT, "nightly patch run")?;
        s.write_string(&P90_MAX_CONCURRENCY, "50")?;
        s.write_boolean(&P90_WITH_DECRYPTION, true)
    }
}

// --- 6 members with @jsonName, for restJson1 ---

member!(
    J_DOCUMENT_NAME,
    "J",
    String,
    "DocumentName",
    0,
    json_name = "documentName"
);
member!(
    J_INSTANCE_ID,
    "J",
    String,
    "InstanceId",
    1,
    json_name = "instanceId"
);
member!(J_TIMEOUT_SECONDS, "J", Integer, "TimeoutSeconds", 2);
member!(J_COMMENT, "J", String, "Comment", 3);
member!(
    J_MAX_CONCURRENCY,
    "J",
    String,
    "MaxConcurrency",
    4,
    json_name = "maxConcurrency"
);
member!(J_WITH_DECRYPTION, "J", Boolean, "WithDecryption", 5);
static J: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "J"),
    ShapeType::Structure,
    &[
        &J_DOCUMENT_NAME,
        &J_INSTANCE_ID,
        &J_TIMEOUT_SECONDS,
        &J_COMMENT,
        &J_MAX_CONCURRENCY,
        &J_WITH_DECRYPTION,
    ],
);

struct JValue;

impl SerializableStruct for JValue {
    fn schema(&self) -> &Schema<'_> {
        &J
    }

    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        s.write_string(&J_DOCUMENT_NAME, "AWS-RunShellScript")?;
        s.write_string(&J_INSTANCE_ID, "i-0123456789abcdef0")?;
        s.write_integer(&J_TIMEOUT_SECONDS, 600)?;
        s.write_string(&J_COMMENT, "nightly patch run")?;
        s.write_string(&J_MAX_CONCURRENCY, "50")?;
        s.write_boolean(&J_WITH_DECRYPTION, true)
    }
}

// --- 20 members (p99) ---

member!(P99_00, "P99", String, "DocumentName", 0);
member!(P99_01, "P99", String, "DocumentVersion", 1);
member!(P99_02, "P99", String, "DocumentHash", 2);
member!(P99_03, "P99", String, "DocumentHashType", 3);
member!(P99_04, "P99", Integer, "TimeoutSeconds", 4);
member!(P99_05, "P99", String, "Comment", 5);
member!(P99_06, "P99", String, "OutputS3Region", 6);
member!(P99_07, "P99", String, "OutputS3BucketName", 7);
member!(P99_08, "P99", String, "OutputS3KeyPrefix", 8);
member!(P99_09, "P99", String, "MaxConcurrency", 9);
member!(P99_10, "P99", String, "MaxErrors", 10);
member!(P99_11, "P99", String, "ServiceRoleArn", 11);
member!(P99_12, "P99", Boolean, "CloudWatchOutputEnabled", 12);
member!(P99_13, "P99", String, "CloudWatchLogGroupName", 13);
member!(P99_14, "P99", String, "NotificationArn", 14);
member!(P99_15, "P99", String, "NotificationType", 15);
member!(P99_16, "P99", Integer, "MaxAttempts", 16);
member!(P99_17, "P99", Timestamp, "ScheduledTime", 17);
member!(P99_18, "P99", Boolean, "ApplyOnlyAtCronInterval", 18);
member!(P99_19, "P99", String, "ClientToken", 19);
static P99: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "P99"),
    ShapeType::Structure,
    &[
        &P99_00, &P99_01, &P99_02, &P99_03, &P99_04, &P99_05, &P99_06, &P99_07, &P99_08, &P99_09,
        &P99_10, &P99_11, &P99_12, &P99_13, &P99_14, &P99_15, &P99_16, &P99_17, &P99_18, &P99_19,
    ],
);

struct P99Value;

impl SerializableStruct for P99Value {
    fn schema(&self) -> &Schema<'_> {
        &P99
    }

    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        s.write_string(&P99_00, "AWS-RunPatchBaseline")?;
        s.write_string(&P99_01, "$DEFAULT")?;
        s.write_string(&P99_02, "9f86d081884c7d659a2feaa0c55ad015")?;
        s.write_string(&P99_03, "Sha256")?;
        s.write_integer(&P99_04, 3600)?;
        s.write_string(&P99_05, "weekly maintenance window")?;
        s.write_string(&P99_06, "us-west-2")?;
        s.write_string(&P99_07, "my-command-output-bucket")?;
        s.write_string(&P99_08, "patching/2026/10")?;
        s.write_string(&P99_09, "10%")?;
        s.write_string(&P99_10, "1")?;
        s.write_string(&P99_11, "arn:aws:iam::123456789012:role/SSMRole")?;
        s.write_boolean(&P99_12, true)?;
        s.write_string(&P99_13, "/aws/ssm/patching")?;
        s.write_string(&P99_14, "arn:aws:sns:us-west-2:123456789012:patching")?;
        s.write_string(&P99_15, "Command")?;
        s.write_integer(&P99_16, 3)?;
        s.write_timestamp(&P99_17, &DateTime::from_secs(1_791_331_200))?;
        s.write_boolean(&P99_18, false)?;
        s.write_string(&P99_19, "6f1c2b9e-1d3a-4c5b-8e7f-0a1b2c3d4e5f")
    }
}

// --- A structure holding a list of 50 tags (102 fields) ---

member!(TAGGED_RESOURCE_ID, "Tagged", String, "ResourceId", 0);
member!(TAGGED_TAGS, "Tagged", List, "Tags", 1);
static TAGGED: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "Tagged"),
    ShapeType::Structure,
    &[&TAGGED_RESOURCE_ID, &TAGGED_TAGS],
);

struct Tagged;

impl SerializableStruct for Tagged {
    fn schema(&self) -> &Schema<'_> {
        &TAGGED
    }

    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        s.write_string(&TAGGED_RESOURCE_ID, "mi-0123456789abcdef0")?;
        s.write_list(&TAGGED_TAGS, &|s| {
            for _ in 0..50 {
                s.write_struct(&TAG, &Tag)?;
            }
            Ok(())
        })
    }
}

// --- restJson1 requests with header bindings, shaped like S3 `PutObject` ---
//
// Most modeled header names are lowercase `x-amz-*` names, with a few
// standard, mixed-case ones (`Cache-Control`, `Content-MD5`).

macro_rules! header {
    ($static_name:ident, $ty:ident, $name:literal, $index:literal, $header:literal) => {
        static $static_name: Schema<'static> = Schema::new_member(
            shape_id!("bench", "Put", $name),
            ShapeType::$ty,
            $name,
            $index,
        )
        .with_http_header($header);
    };
}

static PUT_KEY: Schema<'static> = Schema::new_member(
    shape_id!("bench", "Put", "Key"),
    ShapeType::String,
    "Key",
    0,
)
.with_http_label();
header!(
    PUT_00,
    String,
    "ExpectedBucketOwner",
    1,
    "x-amz-expected-bucket-owner"
);
header!(PUT_01, String, "ACL", 2, "x-amz-acl");
header!(PUT_02, String, "CacheControl", 3, "Cache-Control");
header!(PUT_03, String, "ContentMD5", 4, "Content-MD5");
header!(PUT_04, String, "StorageClass", 5, "x-amz-storage-class");
header!(
    PUT_05,
    String,
    "ServerSideEncryption",
    6,
    "x-amz-server-side-encryption"
);
header!(
    PUT_06,
    String,
    "ContentDisposition",
    7,
    "Content-Disposition"
);
header!(PUT_07, String, "ContentEncoding", 8, "Content-Encoding");
header!(PUT_08, String, "ContentLanguage", 9, "Content-Language");
header!(PUT_09, Timestamp, "Expires", 10, "Expires");
header!(PUT_10, String, "GrantRead", 11, "x-amz-grant-read");
header!(
    PUT_11,
    String,
    "GrantFullControl",
    12,
    "x-amz-grant-full-control"
);
header!(
    PUT_12,
    String,
    "SSEKMSKeyId",
    13,
    "x-amz-server-side-encryption-aws-kms-key-id"
);
header!(
    PUT_13,
    Boolean,
    "BucketKeyEnabled",
    14,
    "x-amz-server-side-encryption-bucket-key-enabled"
);
header!(PUT_14, String, "RequestPayer", 15, "x-amz-request-payer");
header!(PUT_15, String, "Tagging", 16, "x-amz-tagging");
header!(
    PUT_16,
    String,
    "ObjectLockMode",
    17,
    "x-amz-object-lock-mode"
);
header!(
    PUT_17,
    Timestamp,
    "ObjectLockRetainUntilDate",
    18,
    "x-amz-object-lock-retain-until-date"
);
header!(
    PUT_18,
    String,
    "ChecksumAlgorithm",
    19,
    "x-amz-sdk-checksum-algorithm"
);
header!(
    PUT_19,
    Long,
    "WriteOffsetBytes",
    20,
    "x-amz-write-offset-bytes"
);
member!(PUT_DESCRIPTION, "Put", String, "Description", 21);

static PUT: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "Put"),
    ShapeType::Structure,
    &[
        &PUT_KEY,
        &PUT_00,
        &PUT_01,
        &PUT_02,
        &PUT_03,
        &PUT_04,
        &PUT_05,
        &PUT_06,
        &PUT_07,
        &PUT_08,
        &PUT_09,
        &PUT_10,
        &PUT_11,
        &PUT_12,
        &PUT_13,
        &PUT_14,
        &PUT_15,
        &PUT_16,
        &PUT_17,
        &PUT_18,
        &PUT_19,
        &PUT_DESCRIPTION,
    ],
)
.with_http(aws_smithy_schema::traits::HttpTrait::new(
    "PUT", "/{Key}", None,
));

static PUT_HEADERS: [&Schema<'static>; 20] = [
    &PUT_00, &PUT_01, &PUT_02, &PUT_03, &PUT_04, &PUT_05, &PUT_06, &PUT_07, &PUT_08, &PUT_09,
    &PUT_10, &PUT_11, &PUT_12, &PUT_13, &PUT_14, &PUT_15, &PUT_16, &PUT_17, &PUT_18, &PUT_19,
];

/// A `Put` input that sets its first `.0` header members, one URI label and one
/// body member. Like S3 `PutObject`, the input models more headers than a
/// typical request sets.
struct PutValue(usize);

impl SerializableStruct for PutValue {
    fn schema(&self) -> &Schema<'_> {
        &PUT
    }

    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        s.write_string(&PUT_KEY, "photos/2026/10/07/IMG_0001.jpg")?;
        for member in &PUT_HEADERS[..self.0] {
            match member.shape_type() {
                ShapeType::Timestamp => {
                    s.write_timestamp(member, &DateTime::from_secs(1_791_331_200))?
                }
                ShapeType::Boolean => s.write_boolean(member, true)?,
                ShapeType::Long => s.write_long(member, 1_048_576)?,
                _ => s.write_string(member, "bench-value-0123456789")?,
            }
        }
        s.write_string(&PUT_DESCRIPTION, "holiday photo")
    }
}

// --- Benchmarks ---

fn aws_json() -> JsonCodec {
    JsonCodec::new(JsonCodecSettings::builder().use_json_name(false).build())
}

fn rest_json() -> JsonCodec {
    JsonCodec::new(JsonCodecSettings::builder().use_json_name(true).build())
}

fn bench_case(
    c: &mut Criterion,
    name: &str,
    codec: &JsonCodec,
    schema: &'static Schema<'static>,
    value: &dyn SerializableStruct,
) {
    c.bench_function(name, |b| {
        b.iter(|| {
            let mut ser = codec.create_serializer();
            ser.write_struct(black_box(schema), black_box(value))
                .unwrap();
            black_box(ser.finish())
        })
    });
}

fn serialize(c: &mut Criterion) {
    let aws_json = aws_json();
    let rest_json = rest_json();
    bench_case(c, "serialize/awsJson/struct_2", &aws_json, &TAG, &Tag);
    bench_case(c, "serialize/awsJson/struct_6", &aws_json, &P90, &P90Value);
    bench_case(c, "serialize/awsJson/struct_20", &aws_json, &P99, &P99Value);
    bench_case(
        c,
        "serialize/awsJson/list_50_tags",
        &aws_json,
        &TAGGED,
        &Tagged,
    );
    bench_case(
        c,
        "serialize/restJson/struct_6_json_name",
        &rest_json,
        &J,
        &JValue,
    );
}

fn request(c: &mut Criterion) {
    use aws_smithy_json::protocol::aws_rest_json_1::AwsRestJsonProtocol;
    use aws_smithy_schema::protocol::ClientProtocolInner;

    let protocol = AwsRestJsonProtocol::new();
    let cfg = aws_smithy_types::config_bag::ConfigBag::base();
    for headers in [1, 5, 20] {
        let name = format!("request/restJson/headers_{headers}");
        let value = PutValue(headers);
        c.bench_function(&name, |b| {
            b.iter(|| {
                black_box(
                    protocol
                        .serialize_request(
                            black_box(&value),
                            black_box(&PUT),
                            "https://bucket.s3.us-east-1.amazonaws.com",
                            &cfg,
                        )
                        .unwrap(),
                )
            })
        });
    }
}

criterion_group!(benches, serialize, request);
criterion_main!(benches);

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::Credentials;
use aws_sigv4::http_request::{self, SignableBody, SignableRequest, SigningSettings};
use aws_sigv4::sign::v4a;
use aws_smithy_runtime_api::client::identity::Identity;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::time::SystemTime;

const ACCESS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
const SECRET_KEY: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";

const METHOD: &str = "POST";
const URI: &str = "https://abc.lambda-url.us-east-1.on.aws/path?x=1";
const BODY: &[u8] = b"{\"hello\":\"world\"}";

/// Key derivation only: HMAC-SHA256 and the scalar bound check, no EC work.
fn generate_signing_key(c: &mut Criterion) {
    c.bench_function("generate_signing_key", |b| {
        b.iter(|| v4a::generate_signing_key(black_box(ACCESS_KEY), black_box(SECRET_KEY)))
    });
}

/// Signing given an already-derived key, which is the half that does the EC work.
fn calculate_signature(c: &mut Criterion) {
    let key = v4a::generate_signing_key(ACCESS_KEY, SECRET_KEY);
    let string_to_sign =
        b"AWS4-ECDSA-P256-SHA256\n20260101T000000Z\n20260101/lambda/aws4_request\n\
                           a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90";
    c.bench_function("calculate_signature", |b| {
        b.iter(|| v4a::calculate_signature(black_box(&key), black_box(string_to_sign)))
    });
}

/// The whole public entry point, which is what a caller pays per request.
fn sign_http_request(c: &mut Criterion) {
    let identity: Identity = Credentials::new(ACCESS_KEY, SECRET_KEY, None, None, "bench").into();
    // Hoisted: `sign` borrows the params, so building them is setup rather than per-request work.
    // Every field is required by the builder, and `sign` takes the `SigningParams` enum, so this
    // is about as short as it gets. The instant only lands in the credential scope, so the value
    // is arbitrary.
    //
    // `.into()` rather than naming the `V4a` variant: no CI job compiles benches, so a variant
    // rename would break this file silently, where the `From` impl absorbs it. The annotation is
    // only there to pick the `Into` impl.
    let params: http_request::SigningParams<'_> = v4a::SigningParams::builder()
        .identity(&identity)
        .region_set("*")
        .name("lambda")
        .time(SystemTime::UNIX_EPOCH)
        .settings(SigningSettings::default())
        .build()
        .unwrap()
        .into();

    c.bench_function("sign_http_request", |b| {
        b.iter(|| {
            // `sign` takes the request by value, so this one does have to be rebuilt each
            // iteration. It is a handful of borrows and a small Vec, not signing work.
            let request = SignableRequest::new(
                black_box(METHOD),
                black_box(URI),
                [
                    ("host", "abc.lambda-url.us-east-1.on.aws"),
                    ("content-type", "application/json"),
                ]
                .into_iter(),
                SignableBody::Bytes(black_box(BODY)),
            )
            .unwrap();
            http_request::sign(request, &params).unwrap()
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = generate_signing_key, calculate_signature, sign_http_request
}
criterion_main!(benches);

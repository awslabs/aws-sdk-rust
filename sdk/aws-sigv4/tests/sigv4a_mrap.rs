/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg(all(feature = "sigv4a", feature = "http1", feature = "sign-http"))]

use aws_credential_types::Credentials;
use aws_sigv4::http_request::{
    sign, SignableBody, SignableRequest, SignatureLocation, SigningSettings,
};
use aws_sigv4::sign::v4a;
use p256::ecdsa::{signature::Verifier, DerSignature, VerifyingKey};
use sha2::{Digest, Sha256};
use std::time::{Duration, UNIX_EPOCH};

// Static public test credentials only. Signing is entirely offline.
const MRAP: &str = "https://mfzwi23gnjvgw.mrap.accesspoint.s3-global.amazonaws.com/object";
const REGIONS: &str = "us-east-1,us-west-2";

fn verify_published_key(signature: &str, canonical_request: &str) {
    // Coordinates from the published AWS signing suite, not from this KDF.
    let public_key = hex::decode(concat!(
        "04",
        "b6618f6a65740a99e650b33b6b4b5bd0d43b176d721a3edfea7e7d2d56d936b1",
        "865ed22a7eadc9c5cb9d2cbaca1b3699139fedc5043dc6661864218330c8e518"
    ))
    .unwrap();
    let public_key = VerifyingKey::from_sec1_bytes(&public_key).unwrap();
    let signature = DerSignature::try_from(hex::decode(signature).unwrap().as_slice()).unwrap();
    let string_to_sign = format!(
        "AWS4-ECDSA-P256-SHA256\n20260101T000000Z\n20260101/s3/aws4_request\n{}",
        hex::encode(Sha256::digest(canonical_request.as_bytes()))
    );
    public_key
        .verify(string_to_sign.as_bytes(), &signature)
        .unwrap();
    assert!(public_key
        .verify(format!("{string_to_sign}tampered").as_bytes(), &signature)
        .is_err());
}

fn signed_mrap(location: SignatureLocation) -> (http::Request<()>, String) {
    let identity = Credentials::new(
        "AKIDEXAMPLE",
        "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
        None,
        None,
        "test",
    )
    .into();
    let mut settings = SigningSettings::default();
    settings.signature_location = location;
    settings.expires_in = Some(Duration::from_secs(900));
    let params = v4a::SigningParams::builder()
        .identity(&identity)
        .region_set(REGIONS)
        .name("s3")
        .time(UNIX_EPOCH + Duration::from_secs(1_767_225_600))
        .settings(settings)
        .build()
        .unwrap()
        .into();
    let request =
        SignableRequest::new("GET", MRAP, std::iter::empty(), SignableBody::Bytes(&[])).unwrap();
    let (instructions, signature) = sign(request, &params).unwrap().into_parts();
    let mut request = http::Request::builder()
        .method("GET")
        .uri(MRAP)
        .body(())
        .unwrap();
    instructions.apply_to_request_http1x(&mut request);
    p256::ecdsa::DerSignature::try_from(hex::decode(&signature).unwrap().as_slice())
        .expect("MRAP must retain a P-256 DER signature");
    (request, signature)
}

#[test]
fn mrap_header_signing_retains_region_set_and_regionless_scope() {
    let (request, signature) = signed_mrap(SignatureLocation::Headers);
    let authorization = request.headers()["authorization"].to_str().unwrap();
    assert!(authorization
        .starts_with("AWS4-ECDSA-P256-SHA256 Credential=AKIDEXAMPLE/20260101/s3/aws4_request,"));
    assert!(authorization.contains("SignedHeaders=host;x-amz-date;x-amz-region-set"));
    assert_eq!(request.headers()["x-amz-region-set"], REGIONS);
    assert_eq!(request.headers()["x-amz-date"], "20260101T000000Z");
    assert_eq!(
        request.uri().host(),
        Some("mfzwi23gnjvgw.mrap.accesspoint.s3-global.amazonaws.com")
    );
    assert!(request.uri().query().is_none());
    verify_published_key(
        &signature,
        concat!(
            "GET\n/object\n\n",
            "host:mfzwi23gnjvgw.mrap.accesspoint.s3-global.amazonaws.com\n",
            "x-amz-date:20260101T000000Z\n",
            "x-amz-region-set:us-east-1,us-west-2\n\n",
            "host;x-amz-date;x-amz-region-set\n",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        ),
    );
}

#[test]
fn mrap_presigning_retains_region_set_and_regionless_scope() {
    let (request, signature) = signed_mrap(SignatureLocation::QueryParams);
    let query = request.uri().query().unwrap();
    assert!(query.contains("X-Amz-Algorithm=AWS4-ECDSA-P256-SHA256"));
    assert!(query.contains("X-Amz-Credential=AKIDEXAMPLE%2F20260101%2Fs3%2Faws4_request"));
    assert!(query.contains("X-Amz-Region-Set=us-east-1%2Cus-west-2"));
    assert!(query.contains("X-Amz-Expires=900"));
    assert!(query.contains(&format!("X-Amz-Signature={signature}")));
    assert!(!request.headers().contains_key("authorization"));
    verify_published_key(
        &signature,
        concat!(
            "GET\n/object\n",
            "X-Amz-Algorithm=AWS4-ECDSA-P256-SHA256&",
            "X-Amz-Credential=AKIDEXAMPLE%2F20260101%2Fs3%2Faws4_request&",
            "X-Amz-Date=20260101T000000Z&X-Amz-Expires=900&",
            "X-Amz-Region-Set=us-east-1%2Cus-west-2&X-Amz-SignedHeaders=host\n",
            "host:mfzwi23gnjvgw.mrap.accesspoint.s3-global.amazonaws.com\n\n",
            "host\n",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        ),
    );
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::operation::get_object::{GetObject, GetObjectError};
use aws_sdk_s3::operation::list_buckets::ListBuckets;
use aws_sdk_s3::operation::{RequestId, RequestIdExt};
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::operation;
use aws_smithy_http::response::ParseHttpResponse;
use bytes::Bytes;

#[test]
fn get_request_id_from_modeled_error() {
    let resp = http::Response::builder()
        .header("x-amz-request-id", "correct-request-id")
        .header("x-amz-id-2", "correct-extended-request-id")
        .status(404)
        .body(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Error>
              <Code>NoSuchKey</Code>
              <Message>The resource you requested does not exist</Message>
              <Resource>/mybucket/myfoto.jpg</Resource>
              <RequestId>incorrect-request-id</RequestId>
            </Error>"#,
        )
        .unwrap();
    let err = GetObject::new()
        .parse_loaded(&resp.map(Bytes::from))
        .expect_err("status was 404, this is an error");
    assert!(matches!(err, GetObjectError::NoSuchKey(_)));
    assert_eq!(Some("correct-request-id"), err.request_id());
    assert_eq!(Some("correct-request-id"), err.meta().request_id());
    assert_eq!(
        Some("correct-extended-request-id"),
        err.extended_request_id()
    );
    assert_eq!(
        Some("correct-extended-request-id"),
        err.meta().extended_request_id()
    );
}

#[test]
fn get_request_id_from_unmodeled_error() {
    let resp = http::Response::builder()
        .header("x-amz-request-id", "correct-request-id")
        .header("x-amz-id-2", "correct-extended-request-id")
        .status(500)
        .body(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Error>
              <Code>SomeUnmodeledError</Code>
              <Message>Something bad happened</Message>
              <Resource>/mybucket/myfoto.jpg</Resource>
              <RequestId>incorrect-request-id</RequestId>
            </Error>"#,
        )
        .unwrap();
    let err = GetObject::new()
        .parse_loaded(&resp.map(Bytes::from))
        .expect_err("status 500");
    assert!(matches!(err, GetObjectError::Unhandled(_)));
    assert_eq!(Some("correct-request-id"), err.request_id());
    assert_eq!(Some("correct-request-id"), err.meta().request_id());
    assert_eq!(
        Some("correct-extended-request-id"),
        err.extended_request_id()
    );
    assert_eq!(
        Some("correct-extended-request-id"),
        err.meta().extended_request_id()
    );
}

#[test]
fn get_request_id_from_successful_nonstreaming_response() {
    let resp = http::Response::builder()
        .header("x-amz-request-id", "correct-request-id")
        .header("x-amz-id-2", "correct-extended-request-id")
        .status(200)
        .body(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <ListAllMyBucketsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
              <Owner><ID>some-id</ID><DisplayName>some-display-name</DisplayName></Owner>
              <Buckets></Buckets>
            </ListAllMyBucketsResult>"#,
        )
        .unwrap();
    let output = ListBuckets::new()
        .parse_loaded(&resp.map(Bytes::from))
        .expect("valid successful response");
    assert_eq!(Some("correct-request-id"), output.request_id());
    assert_eq!(
        Some("correct-extended-request-id"),
        output.extended_request_id()
    );
}

#[test]
fn get_request_id_from_successful_streaming_response() {
    let resp = http::Response::builder()
        .header("x-amz-request-id", "correct-request-id")
        .header("x-amz-id-2", "correct-extended-request-id")
        .status(200)
        .body(SdkBody::from("some streaming file data"))
        .unwrap();
    let mut resp = operation::Response::new(resp);
    let output = GetObject::new()
        .parse_unloaded(&mut resp)
        .expect("valid successful response");
    assert_eq!(Some("correct-request-id"), output.request_id());
    assert_eq!(
        Some("correct-extended-request-id"),
        output.extended_request_id()
    );
}

// Verify that the conversion from operation error to the top-level service error maintains the request ID
#[test]
fn conversion_to_service_error_maintains_request_id() {
    let resp = http::Response::builder()
        .header("x-amz-request-id", "correct-request-id")
        .header("x-amz-id-2", "correct-extended-request-id")
        .status(404)
        .body(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <Error>
              <Code>NoSuchKey</Code>
              <Message>The resource you requested does not exist</Message>
              <Resource>/mybucket/myfoto.jpg</Resource>
              <RequestId>incorrect-request-id</RequestId>
            </Error>"#,
        )
        .unwrap();
    let err = GetObject::new()
        .parse_loaded(&resp.map(Bytes::from))
        .expect_err("status was 404, this is an error");

    let service_error: aws_sdk_s3::Error = err.into();
    assert_eq!(Some("correct-request-id"), service_error.request_id());
    assert_eq!(
        Some("correct-extended-request-id"),
        service_error.extended_request_id()
    );
}

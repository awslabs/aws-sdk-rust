/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::operation::GetObject;
use aws_sdk_s3::ErrorExt;
use aws_smithy_http::response::ParseHttpResponse;
use bytes::Bytes;

#[test]
fn deserialize_extended_errors() {
    let resp = http::Response::builder()
        .header(
            "x-amz-id-2",
            "gyB+3jRPnrkN98ZajxHXr3u7EFM67bNgSAxexeEHndCX/7GRnfTXxReKUQF28IfP",
        )
        .header("x-amz-request-id", "3B3C7C725673C630")
        .status(404)
        .body(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Error>
  <Code>NoSuchKey</Code>
  <Message>The resource you requested does not exist</Message>
  <Resource>/mybucket/myfoto.jpg</Resource>
  <RequestId>4442587FB7D0A2F9</RequestId>
</Error>"#,
        )
        .unwrap();
    let err = GetObject::new()
        .parse_loaded(&resp.map(Bytes::from))
        .expect_err("status was 404, this is an error");
    assert_eq!(
        err.meta().extended_request_id(),
        Some("gyB+3jRPnrkN98ZajxHXr3u7EFM67bNgSAxexeEHndCX/7GRnfTXxReKUQF28IfP")
    );
    assert_eq!(err.meta().request_id(), Some("4442587FB7D0A2F9"));
}

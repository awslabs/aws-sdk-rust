/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::config::Region;
use aws_sdk_s3::{Client, Config};
use aws_smithy_http_client::test_util::capture_request;
use aws_types::region::SigningRegionSet;

// Verify that a user-configured signing region set is applied during SigV4a signing.

#[tokio::test]
async fn sigv4a_signing_region_set_on_service_config() {
    let (http_client, rx) = capture_request(None);
    let conf = Config::builder()
        .http_client(http_client)
        .region(Region::new("us-east-2"))
        .with_test_defaults()
        .sigv4a_signing_region_set("*")
        .auth_scheme_preference([aws_runtime::auth::sigv4a::SCHEME_ID])
        .build();
    let client = Client::from_conf(conf);
    let _ = client
        .get_object()
        .bucket("arn:aws:s3::123456789012:accesspoint/mfzwi23gnjvgw.mrap")
        .key("test")
        .send()
        .await;
    let req = rx.expect_request();
    let region_set = req
        .headers()
        .get("x-amz-region-set")
        .expect("x-amz-region-set header");
    assert_eq!(region_set, "*");
}

#[tokio::test]
async fn sigv4a_signing_region_set_getter_returns_configured_value() {
    let conf = Config::builder()
        .region(Region::new("us-east-2"))
        .with_test_defaults()
        .sigv4a_signing_region_set("*")
        .build();
    assert_eq!(
        conf.sigv4a_signing_region_set(),
        Some(&SigningRegionSet::from("*"))
    );
}

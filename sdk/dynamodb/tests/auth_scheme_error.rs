/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_dynamodb::config::Region;
use aws_sdk_dynamodb::error::DisplayErrorContext;
use aws_sdk_dynamodb::{Client, Config};
use aws_smithy_http_client::test_util::capture_request;
use aws_smithy_runtime::assert_str_contains;

#[tokio::test]
async fn auth_scheme_error() {
    let (http_client, _) = capture_request(None);
    let config = Config::builder()
        .behavior_version_latest()
        .http_client(http_client)
        .region(Region::new("us-west-2"))
        // intentionally omitting credentials_provider
        .build();
    let client = Client::from_conf(config);

    let err = client
        .list_tables()
        .send()
        .await
        .expect_err("there is no credential provider, so this must fail");
    assert_str_contains!(
        DisplayErrorContext(&err).to_string(),
        "\"sigv4\" wasn't a valid option because there was no identity resolver for it. Be sure to set an identity"
    );
}

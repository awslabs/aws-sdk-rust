/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_iam::Region;

#[tokio::test]
async fn correct_endpoint_resolver() {
    let conf = aws_sdk_iam::Config::builder()
        .region(Region::from_static("iam-fips"))
        .build();
    let operation = aws_sdk_iam::operation::ListRoles::builder()
        .build()
        .unwrap()
        .make_operation(&conf)
        .await
        .expect("valid operation");
    let props = operation.properties();
    let ep: &aws_smithy_http::endpoint::Result =
        props.get().expect("endpoint result was not present");
    let ep = ep.as_ref().expect("ep resolved successfully");
    // test fips endpoint
    assert_eq!(ep.url(), "https://iam-fips.amazonaws.com/");
}

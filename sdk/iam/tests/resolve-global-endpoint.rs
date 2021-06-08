/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_endpoint::get_endpoint_resolver;
use aws_sdk_iam::Region;
use http::Uri;

#[test]
fn correct_endpoint_resolver() {
    let conf = aws_sdk_iam::Config::builder().build();
    let operation = aws_sdk_iam::operation::ListRoles::builder()
        .build()
        .unwrap()
        .make_operation(&conf)
        .expect("valid operation");
    let conf = operation.config();
    let resolver = get_endpoint_resolver(&conf).expect("operation should have endpoint resolver");
    // test regular endpoint
    {
        let ep = resolver
            .resolve_endpoint(&Region::new("us-east-1"))
            .expect("valid endpoint");
        let mut uri = Uri::from_static("/");
        ep.set_endpoint(&mut uri, None);
        assert_eq!(uri, Uri::from_static("https://iam.amazonaws.com/"));
    }
    // test fips endpoint
    {
        let ep = resolver
            .resolve_endpoint(&Region::new("iam-fips"))
            .expect("valid endpoint");
        let mut uri = Uri::from_static("/");
        ep.set_endpoint(&mut uri, None);
        assert_eq!(uri, Uri::from_static("https://iam-fips.amazonaws.com/"));
    }
}

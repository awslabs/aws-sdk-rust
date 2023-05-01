/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::body::SdkBody;

pub(crate) fn make_request(_req: http::Request<SdkBody>) -> Result<http::Response<SdkBody>, ()> {
    // Consumers here would pass the HTTP request to
    // the Wasm host in order to get the response back
    let body = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
    <ListAllMyBucketsResult>
    <Buckets>
        <Bucket>
            <CreationDate>2023-01-23T11:59:03.575496Z</CreationDate>
            <Name>doc-example-bucket</Name>
        </Bucket>
        <Bucket>
            <CreationDate>2023-01-23T23:32:13.125238Z</CreationDate>
            <Name>doc-example-bucket2</Name>
        </Bucket>
    </Buckets>
    <Owner>
        <DisplayName>account-name</DisplayName>
        <ID>a3a42310-42d0-46d1-9745-0cee9f4fb851</ID>
    </Owner>
    </ListAllMyBucketsResult>";
    Ok(http::Response::new(SdkBody::from(body)))
}

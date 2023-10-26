/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::body::SdkBody;
use aws_smithy_runtime_api::client::orchestrator::{
    BoxFuture, Connection, Future, HttpRequest, HttpResponse,
};

#[derive(Debug, Default)]
pub struct OkConnector {}

impl OkConnector {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Connection for OkConnector {
    fn call(&self, _request: HttpRequest) -> BoxFuture<HttpResponse> {
        Box::pin(Future::ready(Ok(http::Response::builder()
            .status(200)
            .body(SdkBody::empty())
            .expect("OK response is valid"))))
    }
}

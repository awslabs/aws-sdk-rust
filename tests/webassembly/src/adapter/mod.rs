/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod http_client;

use aws_smithy_client::erase::DynConnector;
use aws_smithy_client::http_connector::HttpConnector;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::ConnectorError;
use std::task::{Context, Poll};
use tower::Service;

#[derive(Default, Debug, Clone)]
pub(crate) struct Adapter {}

impl Adapter {
    pub fn to_http_connector() -> impl Into<HttpConnector> {
        DynConnector::new(Adapter::default())
    }
}

impl Service<http::Request<SdkBody>> for Adapter {
    type Response = http::Response<SdkBody>;

    type Error = ConnectorError;

    #[allow(clippy::type_complexity)]
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>,
    >;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<SdkBody>) -> Self::Future {
        println!("Adapter: sending request...");
        let res = http_client::make_request(req).unwrap();
        println!("{:?}", res);
        Box::pin(async move { Ok(res) })
    }
}

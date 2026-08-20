/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Client configuration and request helpers shared by connection behavior tests.

use aws_smithy_async::rt::sleep::{SharedAsyncSleep, TokioSleep};
use aws_smithy_async::time::SystemTimeSource;
use aws_smithy_runtime_api::client::http::{
    HttpClient, HttpConnector, HttpConnectorSettings, SharedHttpClient, SharedHttpConnector,
};
use aws_smithy_runtime_api::client::orchestrator::{HttpRequest, HttpResponse};
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_runtime_api::client::runtime_components::{
    RuntimeComponents, RuntimeComponentsBuilder,
};
use http_body_util::BodyExt;
use std::time::Duration;

/// Default timeout for test waits and deadline assertions.
pub(crate) const WAIT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BackendConfig {
    pub(crate) pool_idle_timeout: Option<Duration>,
}

/// Hyper 1.x through `hyper_util::client::legacy::Client`.
///
/// "Legacy" is Hyper Util's module name and does not refer to smithy-rs's `hyper-014` feature.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HyperUtilLegacyPool;

pub(crate) fn runtime_components() -> RuntimeComponents {
    RuntimeComponentsBuilder::for_tests()
        .with_time_source(Some(SystemTimeSource::new()))
        .with_sleep_impl(Some(SharedAsyncSleep::new(TokioSleep::new())))
        .build()
        .expect("valid runtime components")
}

pub(crate) fn connector_with_settings(
    client: &SharedHttpClient,
    settings: HttpConnectorSettings,
) -> SharedHttpConnector {
    client.http_connector(&settings, &runtime_components())
}

pub(crate) fn connector(client: &SharedHttpClient) -> SharedHttpConnector {
    connector_with_settings(client, HttpConnectorSettings::builder().build())
}

pub(crate) async fn send_request(
    connector: &SharedHttpConnector,
    request: HttpRequest,
) -> Result<HttpResponse, ConnectorError> {
    tokio::time::timeout(WAIT, connector.call(request))
        .await
        .expect("request should finish within the outer deadline")
}

pub(crate) async fn send_and_collect(
    connector: &SharedHttpConnector,
    request: HttpRequest,
) -> (u16, Vec<u8>) {
    let response = send_request(connector, request)
        .await
        .expect("request should succeed");
    collect_response(response).await
}

pub(crate) async fn get_and_collect(connector: &SharedHttpConnector, url: &str) -> (u16, Vec<u8>) {
    send_and_collect(
        connector,
        HttpRequest::get(url).expect("valid HTTP request"),
    )
    .await
}

pub(crate) async fn collect_response(response: HttpResponse) -> (u16, Vec<u8>) {
    let status = response.status().as_u16();
    let body = tokio::time::timeout(WAIT, response.into_body().collect())
        .await
        .expect("response body should finish within the outer deadline")
        .expect("response body should be readable")
        .to_bytes()
        .to_vec();
    (status, body)
}

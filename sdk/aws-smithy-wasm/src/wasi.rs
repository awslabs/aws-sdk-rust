/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! WASI HTTP Adapter
use aws_smithy_async::rt::sleep::{AsyncSleep, Sleep};
use aws_smithy_runtime_api::client::connector_metadata::ConnectorMetadata;
use aws_smithy_runtime_api::{
    client::{
        http::{
            HttpClient, HttpConnector, HttpConnectorFuture, HttpConnectorSettings,
            SharedHttpClient, SharedHttpConnector,
        },
        orchestrator::HttpRequest,
        result::ConnectorError,
        runtime_components::RuntimeComponents,
    },
    http::Response,
    shared::IntoShared,
};
use aws_smithy_types::{body::SdkBody, retry::ErrorKind};
use http_body_util::{BodyStream, StreamBody};
use std::time::Duration;
use wstd::http::{Body as WstdBody, BodyExt as _, Client, Error as WstdError};

/// An sleep implementation for wasip2, using the wstd async executor.
#[derive(Debug, Clone)]
pub struct WasiSleep;
impl AsyncSleep for WasiSleep {
    fn sleep(&self, duration: Duration) -> Sleep {
        Sleep::new(async move {
            wstd::task::sleep(wstd::time::Duration::from(duration)).await;
        })
    }
}

/// Builder for [`WasiHttpClient`]. Currently empty, but allows for future
/// config options to be added in a backwards compatible manner.
#[derive(Default, Debug)]
#[non_exhaustive]
pub struct WasiHttpClientBuilder {}

impl WasiHttpClientBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Builds the [`WasiHttpClient`].
    pub fn build(self) -> SharedHttpClient {
        let client = WasiHttpClient {};
        client.into_shared()
    }
}

/// An HTTP client that can be used during instantiation of the client SDK in
/// order to route the HTTP requests through the WebAssembly host. The host must
/// support the wasi-http interface as defined in the WASIp2 specification.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct WasiHttpClient {}

impl HttpClient for WasiHttpClient {
    fn http_connector(
        &self,
        settings: &HttpConnectorSettings,
        _components: &RuntimeComponents,
    ) -> SharedHttpConnector {
        let mut client = Client::new();
        if let Some(timeout) = settings.connect_timeout() {
            client.set_connect_timeout(timeout);
        }
        if let Some(timeout) = settings.read_timeout() {
            client.set_first_byte_timeout(timeout);
        }
        SharedHttpConnector::new(WasiHttpConnector(client))
    }

    fn connector_metadata(&self) -> Option<ConnectorMetadata> {
        Some(ConnectorMetadata::new("wasi-http-client", None))
    }
}

/// HTTP connector used in WASI environment
#[derive(Debug, Clone)]
struct WasiHttpConnector(Client);

impl HttpConnector for WasiHttpConnector {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        let client = self.0.clone();
        HttpConnectorFuture::new(async move {
            let request = request
                .try_into_http1x()
                // This can only fail if the Extensions fail to convert
                .map_err(|e| ConnectorError::other(Box::new(e), None))?;
            // smithy's SdkBody Error is a non-'static boxed dyn stderror.
            // Anyhow can't represent that, so convert it to the debug impl.
            let request = request.map(|body| {
                WstdBody::from_http_body(body.map_err(|e| WstdError::msg(format!("{e:?}"))))
            });
            // Any error given by send is considered a "ClientError" kind
            // which should prevent smithy from retrying like it would for a
            // throttling error
            let response = client
                .send(request)
                .await
                .map_err(|e| ConnectorError::other(e.into(), Some(ErrorKind::ClientError)))?;

            Response::try_from(response.map(|wstd_body| {
                // You'd think that an SdkBody would just be an impl Body with
                // the usual error type dance.
                let nonsync_body = wstd_body
                    .into_boxed_body()
                    .map_err(|e| e.into_boxed_dyn_error());
                // But we have to do this weird dance: because Axum insists
                // bodies are not Sync, wstd settled on non-Sync bodies.
                // Smithy insists on Sync bodies. The SyncStream type exists
                // to assert, because all Stream operations are on &mut self,
                // all Streams are Sync. So, turn the Body into a Stream, make
                // it sync, then back to a Body.
                let nonsync_stream = BodyStream::new(nonsync_body);
                let sync_stream = sync_wrapper::SyncStream::new(nonsync_stream);
                let sync_body = StreamBody::new(sync_stream);
                SdkBody::from_body_1_x(sync_body)
            }))
            // This can only fail if the Extensions fail to convert
            .map_err(|e| ConnectorError::other(Box::new(e), None))
        })
    }
}

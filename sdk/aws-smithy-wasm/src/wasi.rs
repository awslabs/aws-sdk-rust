/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! WASI HTTP Adapter
use aws_smithy_http::header::ParseError;
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
use aws_smithy_types::body::SdkBody;
use bytes::{Bytes, BytesMut};
use wasip2::http::{
    outgoing_handler,
    types::{self as wasi_http, OutgoingBody, RequestOptions},
};

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
/// support the WASI HTTP proposal as defined in the Preview 2 specification.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct WasiHttpClient {}

impl HttpClient for WasiHttpClient {
    fn http_connector(
        &self,
        settings: &HttpConnectorSettings,
        _components: &RuntimeComponents,
    ) -> SharedHttpConnector {
        let options = WasiRequestOptions::from(settings);
        let connector = WasiHttpConnector { options };

        connector.into_shared()
    }

    fn connector_metadata(&self) -> Option<ConnectorMetadata> {
        Some(ConnectorMetadata::new("wasi-http-client", None))
    }
}

/// HTTP connector used in WASI environment
#[derive(Debug, Clone)]
struct WasiHttpConnector {
    options: WasiRequestOptions,
}

impl HttpConnector for WasiHttpConnector {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        tracing::trace!("WasiHttpConnector: sending request {request:?}");

        let client = WasiDefaultClient::new(self.options.clone());
        let http_req = request.try_into_http1x().expect("Http request invalid");
        let converted_req = http_req.map(|body| match body.bytes() {
            Some(value) => Bytes::copy_from_slice(value),
            None => Bytes::new(),
        });

        let fut_result = client.handle(converted_req);

        HttpConnectorFuture::new(async move {
            let fut = fut_result?;
            let response = fut.map(|body| {
                if body.is_empty() {
                    SdkBody::empty()
                } else {
                    SdkBody::from(body)
                }
            });
            tracing::trace!("WasiHttpConnector: response received {response:?}");

            let sdk_res = Response::try_from(response)
                .map_err(|err| ConnectorError::other(err.into(), None))?;

            Ok(sdk_res)
        })
    }
}

/// WASI HTTP client containing the options passed to the outgoing_handler
struct WasiDefaultClient {
    options: WasiRequestOptions,
}

impl WasiDefaultClient {
    /// Create a new WASI HTTP client.
    fn new(options: WasiRequestOptions) -> Self {
        Self { options }
    }

    /// Make outgoing HTTP request in a WASI environment
    fn handle(&self, req: http::Request<Bytes>) -> Result<http::Response<Bytes>, ConnectorError> {
        let req =
            WasiRequest::try_from(req).map_err(|err| ConnectorError::other(err.into(), None))?;

        let res = outgoing_handler::handle(req.0, self.options.clone().0)
            .map_err(|err| ConnectorError::other(err.into(), None))?;

        // Right now only synchronous calls can be made through WASI, so we subscribe and
        // block on the FutureIncomingResponse
        let subscription = res.subscribe();
        subscription.block();

        //The FutureIncomingResponse .get() method returns a
        //Option<Result<Result<IncomingResponse, ErrorCode>, ()>>.
        //The outer Option ensures readiness which we know is Some because we .block() waiting for it
        //The outer Result is just a singleton enforcer so we can only get the response once
        //The inner Result indicates whether the HTTP call was sent/received successfully (not the 200 succes of the call)
        let incoming_res = res
            .get()
            .expect("Http response not ready")
            .expect("Http response accessed more than once")
            .map_err(|err| ConnectorError::other(err.into(), None))?;

        let response = http::Response::try_from(WasiResponse(incoming_res))
            .map_err(|err| ConnectorError::other(err.into(), None))?;

        Ok(response)
    }
}

/// Wrapper for the WASI RequestOptions type to allow us to impl Clone
#[derive(Debug)]
struct WasiRequestOptions(Option<outgoing_handler::RequestOptions>);

impl From<&HttpConnectorSettings> for WasiRequestOptions {
    fn from(value: &HttpConnectorSettings) -> Self {
        //The WASI Duration is nanoseconds represented as u64
        //Note: that the HttpConnectorSettings provides nanoseconds as u128
        //so here we are clamping to u64::MAX if the value is above that
        let connect_timeout = value
            .connect_timeout()
            .map(|dur| u64::try_from(dur.as_nanos()).unwrap_or(u64::MAX));
        let read_timeout = value
            .read_timeout()
            .map(|dur| u64::try_from(dur.as_nanos()).unwrap_or(u64::MAX));

        //Note: these only fail if setting this particular type of timeout is not
        //supported. Spec compliant runtimes should always support these so it is
        //unlikely to be an issue.
        let wasi_http_opts = wasi_http::RequestOptions::new();
        wasi_http_opts
            .set_connect_timeout(connect_timeout)
            .expect("Connect timeout not supported");
        wasi_http_opts
            .set_first_byte_timeout(read_timeout)
            .expect("Read timeout not supported");

        WasiRequestOptions(Some(wasi_http_opts))
    }
}
//The WASI RequestOptions type doesn't impl copy or clone but the outgoing_handler::handle method
//takes ownership, so we impl it on this wrapper type
impl Clone for WasiRequestOptions {
    fn clone(&self) -> Self {
        //Note none of the expects here should ever trigger since all of the values passed in are from
        //the existing RequestOptions that is being cloned and should be valid
        let new_opts = if let Some(opts) = &self.0 {
            let new_opts = RequestOptions::new();
            new_opts
                .set_between_bytes_timeout(opts.between_bytes_timeout())
                .expect("Between bytes timeout");
            new_opts
                .set_connect_timeout(opts.connect_timeout())
                .expect("Connect timeout");
            new_opts
                .set_first_byte_timeout(opts.first_byte_timeout())
                .expect("First byte timeout");

            Some(new_opts)
        } else {
            None
        };

        Self(new_opts)
    }
}

/// Wrapper to allow converting between HTTP Request types and WASI Request types
#[derive(Debug)]
struct WasiRequest(outgoing_handler::OutgoingRequest);

impl TryFrom<http::Request<Bytes>> for WasiRequest {
    type Error = ParseError;

    fn try_from(value: http::Request<Bytes>) -> Result<Self, Self::Error> {
        let (parts, body) = value.into_parts();
        let method = WasiMethod::try_from(parts.method)?;
        let path_with_query = parts.uri.path_and_query().map(|path| path.as_str());
        let headers = WasiHeaders::try_from(parts.headers)?;
        let scheme = match parts.uri.scheme_str().unwrap_or("") {
            "http" => Some(&wasi_http::Scheme::Http),
            "https" => Some(&wasi_http::Scheme::Https),
            _ => None,
        };
        let authority = parts.uri.authority().map(|auth| auth.as_str());

        let request = wasi_http::OutgoingRequest::new(headers.0);
        request
            .set_scheme(scheme)
            .map_err(|_| ParseError::new("Failed to set HTTP scheme"))?;
        request
            .set_method(&method.0)
            .map_err(|_| ParseError::new("Failed to set HTTP method"))?;
        request
            .set_path_with_query(path_with_query)
            .map_err(|_| ParseError::new("Failed to set HTTP path"))?;
        request
            .set_authority(authority)
            .map_err(|_| ParseError::new("Failed to set HTTP authority"))?;

        let request_body = request.body().expect("Body accessed more than once");

        let request_stream = request_body
            .write()
            .expect("Output stream accessed more than once");

        request_stream
            .blocking_write_and_flush(&body)
            .map_err(|_| ParseError::new("Failed to write HTTP body"))?;

        //The OutputStream is a child resource: it must be dropped
        //before the parent OutgoingBody resource is dropped (or finished),
        //otherwise the OutgoingBody drop or finish will trap.
        drop(request_stream);

        OutgoingBody::finish(request_body, None)
            .map_err(|_| ParseError::new("Failed to finalize HTTP body"))?;

        Ok(WasiRequest(request))
    }
}

/// Wrapper to allow converting between HTTP Methods and WASI Methods
struct WasiMethod(wasi_http::Method);

impl TryFrom<http::Method> for WasiMethod {
    type Error = ParseError;

    fn try_from(method: http::Method) -> Result<Self, Self::Error> {
        Ok(Self(match method {
            http::Method::GET => wasi_http::Method::Get,
            http::Method::POST => wasi_http::Method::Post,
            http::Method::PUT => wasi_http::Method::Put,
            http::Method::DELETE => wasi_http::Method::Delete,
            http::Method::PATCH => wasi_http::Method::Patch,
            http::Method::CONNECT => wasi_http::Method::Connect,
            http::Method::TRACE => wasi_http::Method::Trace,
            http::Method::HEAD => wasi_http::Method::Head,
            http::Method::OPTIONS => wasi_http::Method::Options,
            _ => return Err(ParseError::new("failed due to unsupported method, currently supported methods are: GET, POST, PUT, DELETE, PATCH, CONNECT, TRACE, HEAD, and OPTIONS")),
        }))
    }
}

/// Wrapper to allow converting between HTTP Response types and WASI Response types
struct WasiResponse(wasi_http::IncomingResponse);

impl TryFrom<WasiResponse> for http::Response<Bytes> {
    type Error = ParseError;

    fn try_from(value: WasiResponse) -> Result<Self, Self::Error> {
        let response = value.0;

        let status = response.status();

        //This headers resource is a child: it must be dropped before the parent incoming-response is dropped.
        //The drop happens via the consuming iterator used below
        let headers = response.headers().entries();

        let res_build = headers
            .into_iter()
            .fold(http::Response::builder().status(status), |rb, header| {
                rb.header(header.0, header.1)
            });

        let body_incoming = response.consume().expect("Consume called more than once");

        //The input-stream resource is a child: it must be dropped before the parent
        //incoming-body is dropped, or consumed by incoming-body.finish.
        //That drop is done explicitly below
        let body_stream = body_incoming
            .stream()
            .expect("Stream accessed more than once");

        let mut body = BytesMut::new();

        //blocking_read blocks until at least one byte is available
        while let Ok(stream_bytes) = body_stream.blocking_read(u64::MAX) {
            body.extend_from_slice(stream_bytes.as_slice())
        }

        drop(body_stream);

        let res = res_build
            .body(body.freeze())
            .map_err(|err| ParseError::new(err.to_string()))?;

        Ok(res)
    }
}

/// Wrapper to allow converting between HTTP headers and WASI headers
struct WasiHeaders(wasi_http::Fields);

impl TryFrom<http::HeaderMap> for WasiHeaders {
    type Error = ParseError;

    fn try_from(headers: http::HeaderMap) -> Result<Self, Self::Error> {
        let entries = headers
            .iter()
            .map(|(name, value)| {
                (
                    name.to_string(),
                    value.to_str().unwrap().as_bytes().to_vec(),
                )
            })
            .collect::<Vec<_>>();

        let fields = wasi_http::Fields::from_list(&entries)
            .map_err(|err| ParseError::new(err.to_string()))?;

        Ok(Self(fields))
    }
}

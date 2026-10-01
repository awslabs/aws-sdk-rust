/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
use crate::client::connect;
use crate::proxy;
use aws_smithy_runtime_api::box_error::BoxError;
use http_1x::{Extensions, HeaderMap, HeaderValue, Uri};
use hyper::rt::{Read, ReadBufCursor, Write};
use hyper_util::client::legacy::connect::{Connected, Connection};
use hyper_util::client::proxy::matcher::Matcher;
use pin_project_lite::pin_project;
use std::fmt;
use std::future::Future;
use std::io;
use std::io::IoSlice;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub(crate) trait AsyncConn:
    Read + Write + Connection + Send + Sync + Unpin + 'static
{
}

impl<T: Read + Write + Connection + Send + Sync + Unpin + 'static> AsyncConn for T {}

pub(crate) type BoxConn = Box<dyn AsyncConn>;

// Future for connecting
pub(crate) type Connecting = Pin<Box<dyn Future<Output = Result<Conn, BoxError>> + Send>>;

/// Authorization selected for cleartext HTTP requests sent through a proxy.
#[derive(Clone)]
pub(crate) struct ProxyAuthorization(HeaderValue);

impl fmt::Debug for ProxyAuthorization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProxyAuthorization(** redacted **)")
    }
}

impl ProxyAuthorization {
    pub(crate) fn new(value: HeaderValue) -> Self {
        Self(value)
    }

    /// Applies the selected authorization unless the caller supplied one.
    pub(crate) fn apply(&self, headers: &mut HeaderMap) {
        headers
            .entry(http_1x::header::PROXY_AUTHORIZATION)
            .or_insert_with(|| self.0.clone());
    }
}

/// How an established transport reaches its origin.
///
/// The classification contains no proxy credentials. It describes the path
/// selected by the built-in connector or reported by a custom connector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConnectPath {
    /// The transport connects to the origin without an HTTP proxy.
    Direct,
    /// HTTP requests are sent to a forward proxy in absolute form.
    ForwardProxy,
    /// The connector established an HTTP `CONNECT` tunnel to the origin.
    ProxyTunnel,
}

impl ConnectPath {
    /// Returns whether an HTTP proxy participates in the connection path.
    pub fn is_proxied(&self) -> bool {
        !matches!(self, Self::Direct)
    }
}

/// Connector-owned connection path with request-time proxy state.
#[derive(Clone, Debug)]
pub(crate) enum ConnectPathInner {
    /// The transport reaches the origin without a configured HTTP proxy.
    Direct,
    /// Requests are sent to a forward proxy; HTTP/1 uses absolute-form targets.
    ForwardProxy {
        authorization: Option<ProxyAuthorization>,
    },
    /// The transport reaches the origin through an established CONNECT tunnel.
    ProxyTunnel,
}

impl ConnectPathInner {
    pub(crate) fn forward_proxy(authorization: Option<HeaderValue>) -> Self {
        Self::ForwardProxy {
            authorization: authorization.map(ProxyAuthorization::new),
        }
    }

    /// Recovers the full path when available and preserves custom connector behavior.
    pub(crate) fn from_connected(connected: &Connected, extras: &Extensions) -> Self {
        extras
            .get::<Self>()
            .cloned()
            .or_else(|| extras.get::<ConnectPath>().copied().map(Self::from_public))
            .unwrap_or_else(|| {
                if connected.is_proxied() {
                    Self::forward_proxy(None)
                } else {
                    Self::Direct
                }
            })
    }

    /// Returns the credential-free public path classification.
    pub(crate) fn public(&self) -> ConnectPath {
        match self {
            Self::Direct => ConnectPath::Direct,
            Self::ForwardProxy { .. } => ConnectPath::ForwardProxy,
            Self::ProxyTunnel => ConnectPath::ProxyTunnel,
        }
    }

    /// Creates connector state from a credential-free custom-connector value.
    fn from_public(path: ConnectPath) -> Self {
        match path {
            ConnectPath::Direct => Self::Direct,
            ConnectPath::ForwardProxy => Self::forward_proxy(None),
            ConnectPath::ProxyTunnel => Self::ProxyTunnel,
        }
    }

    /// Returns whether HTTP/1 requests require an absolute-form target.
    pub(crate) fn uses_absolute_form(&self) -> bool {
        matches!(self, Self::ForwardProxy { .. })
    }

    /// Applies configured forwarding credentials unless the request supplied them.
    pub(crate) fn apply_proxy_authorization(&self, headers: &mut HeaderMap) {
        if let Self::ForwardProxy {
            authorization: Some(authorization),
        } = self
        {
            authorization.apply(headers);
        }
    }
}

pin_project! {
    pub(crate) struct Conn {
        #[pin]
        pub(super)inner: BoxConn,
        pub(super) connect_path: ConnectPathInner,
    }
}

impl Connection for Conn {
    fn connected(&self) -> Connected {
        self.inner
            .connected()
            .proxy(self.connect_path.uses_absolute_form())
            .extra(self.connect_path.clone())
    }
}

impl Read for Conn {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: ReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.project();
        Read::poll_read(this.inner, cx, buf)
    }
}

impl Write for Conn {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let this = self.project();
        Write::poll_write(this.inner, cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize, io::Error>> {
        let this = self.project();
        Write::poll_write_vectored(this.inner, cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let this = self.project();
        Write::poll_flush(this.inner, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let this = self.project();
        Write::poll_shutdown(this.inner, cx)
    }
}

/// HTTP-only proxy connector for handling HTTP requests through HTTP proxies
///
/// This connector handles the HTTP proxy logic when no TLS provider is selected,
/// including request URL modification and proxy authentication.
#[derive(Debug, Clone)]
pub(crate) struct HttpProxyConnector<C> {
    inner: C,
    proxy_matcher: Option<Arc<Matcher>>,
}

impl<C> HttpProxyConnector<C> {
    pub(crate) fn new(inner: C, proxy_config: proxy::ProxyConfig) -> Self {
        let proxy_matcher = if proxy_config.is_disabled() {
            None
        } else {
            Some(Arc::new(proxy_config.into_hyper_util_matcher()))
        };
        Self {
            inner,
            proxy_matcher,
        }
    }
}

impl<C> tower::Service<Uri> for HttpProxyConnector<C>
where
    C: tower::Service<Uri> + Clone + Send + 'static,
    C::Response: hyper::rt::Read
        + hyper::rt::Write
        + hyper_util::client::legacy::connect::Connection
        + Send
        + Sync
        + Unpin
        + 'static,
    C::Future: Send + 'static,
    C::Error: Into<BoxError>,
{
    type Response = connect::Conn;
    type Error = BoxError;
    type Future = connect::Connecting;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, dst: Uri) -> Self::Future {
        let proxy_intercept = self
            .proxy_matcher
            .as_ref()
            .and_then(|matcher| matcher.intercept(&dst));

        if let Some(intercept) = proxy_intercept {
            // HTTP through proxy: Connect to proxy server
            let proxy_uri = intercept.uri().clone();
            let connect_path = ConnectPathInner::forward_proxy(intercept.basic_auth().cloned());
            let fut = self.inner.call(proxy_uri);
            Box::pin(async move {
                let conn = fut.await.map_err(Into::into)?;
                Ok(connect::Conn {
                    inner: Box::new(conn),
                    connect_path,
                })
            })
        } else {
            // Direct connection
            let fut = self.inner.call(dst);
            Box::pin(async move {
                let conn = fut.await.map_err(Into::into)?;
                Ok(connect::Conn {
                    inner: Box::new(conn),
                    connect_path: ConnectPathInner::Direct,
                })
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_connector_path_extra_preserves_public_classification() {
        for path in [
            ConnectPath::Direct,
            ConnectPath::ForwardProxy,
            ConnectPath::ProxyTunnel,
        ] {
            let connected = Connected::new().extra(path);
            let mut extras = Extensions::new();
            connected.get_extras(&mut extras);

            assert_eq!(
                path,
                ConnectPathInner::from_connected(&connected, &extras).public()
            );
        }
    }

    #[test]
    fn generic_proxy_metadata_maps_to_forward_proxy() {
        let connected = Connected::new().proxy(true);
        let mut extras = Extensions::new();
        connected.get_extras(&mut extras);

        assert_eq!(
            ConnectPath::ForwardProxy,
            ConnectPathInner::from_connected(&connected, &extras).public()
        );
    }
}

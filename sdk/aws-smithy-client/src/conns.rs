/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Type aliases for standard connection types.

#[cfg(feature = "rustls")]
/// A `hyper` connector that uses the `rustls` crate for TLS. To use this in a smithy client,
/// wrap it in a [hyper_ext::Adapter](crate::hyper_ext::Adapter).
pub type Https = hyper_rustls::HttpsConnector<hyper::client::HttpConnector>;

#[cfg(feature = "native-tls")]
/// A `hyper` connector that uses the `native-tls` crate for TLS. To use this in a smithy client,
/// wrap it in a [hyper_ext::Adapter](crate::hyper_ext::Adapter).
pub type NativeTls = hyper_tls::HttpsConnector<hyper::client::HttpConnector>;

#[cfg(feature = "rustls")]
/// A smithy connector that uses the `rustls` crate for TLS.
pub type Rustls = crate::hyper_ext::Adapter<Https>;

// Creating a `with_native_roots` HTTP client takes 300ms on OS X. Cache this so that we
// don't need to repeatedly incur that cost.
#[cfg(feature = "rustls")]
lazy_static::lazy_static! {
    static ref HTTPS_NATIVE_ROOTS: Https = {
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build()
    };
}

#[cfg(feature = "rustls")]
/// Return a default HTTPS connector backed by the `rustls` crate.
///
/// It requires a minimum TLS version of 1.2.
/// It allows you to connect to both `http` and `https` URLs.
pub fn https() -> Https {
    HTTPS_NATIVE_ROOTS.clone()
}

#[cfg(feature = "native-tls")]
/// Return a default HTTPS connector backed by the `hyper_tls` crate.
///
/// It requires a minimum TLS version of 1.2.
/// It allows you to connect to both `http` and `https` URLs.
pub fn native_tls() -> NativeTls {
    // `TlsConnector` actually comes for here: https://docs.rs/native-tls/latest/native_tls/
    // hyper_tls just re-exports the crate for convenience.
    let mut tls = hyper_tls::native_tls::TlsConnector::builder();
    let tls = tls
        .min_protocol_version(Some(hyper_tls::native_tls::Protocol::Tlsv12))
        .build()
        .unwrap_or_else(|e| panic!("Error while creating TLS connector: {}", e));
    let mut http = hyper::client::HttpConnector::new();
    http.enforce_http(false);
    hyper_tls::HttpsConnector::from((http, tls.into()))
}

#[cfg(all(test, any(feature = "native-tls", feature = "rustls")))]
mod tests {
    use crate::erase::DynConnector;
    use crate::hyper_ext::Adapter;
    use aws_smithy_http::body::SdkBody;
    use http::{Method, Request, Uri};
    use tower::{Service, ServiceBuilder};

    async fn send_request_and_assert_success(conn: DynConnector, uri: &Uri) {
        let mut svc = ServiceBuilder::new().service(conn);
        let req = Request::builder()
            .uri(uri)
            .method(Method::GET)
            .body(SdkBody::empty())
            .unwrap();
        let res = svc.call(req).await.unwrap();
        assert!(res.status().is_success());
    }

    #[cfg(feature = "native-tls")]
    mod native_tls_tests {
        use super::super::native_tls;
        use super::*;

        #[tokio::test]
        async fn test_native_tls_connector_can_make_http_requests() {
            let conn = Adapter::builder().build(native_tls());
            let conn = DynConnector::new(conn);
            let http_uri: Uri = "http://example.com/".parse().unwrap();

            send_request_and_assert_success(conn, &http_uri).await;
        }

        #[tokio::test]
        async fn test_native_tls_connector_can_make_https_requests() {
            let conn = Adapter::builder().build(native_tls());
            let conn = DynConnector::new(conn);
            let https_uri: Uri = "https://example.com/".parse().unwrap();

            send_request_and_assert_success(conn, &https_uri).await;
        }
    }

    #[cfg(feature = "rustls")]
    mod rustls_tests {
        use super::super::https;
        use super::*;

        #[tokio::test]
        async fn test_rustls_connector_can_make_http_requests() {
            let conn = Adapter::builder().build(https());
            let conn = DynConnector::new(conn);
            let http_uri: Uri = "http://example.com/".parse().unwrap();

            send_request_and_assert_success(conn, &http_uri).await;
        }

        #[tokio::test]
        async fn test_rustls_connector_can_make_https_requests() {
            let conn = Adapter::builder().build(https());
            let conn = DynConnector::new(conn);
            let https_uri: Uri = "https://example.com/".parse().unwrap();

            send_request_and_assert_success(conn, &https_uri).await;
        }
    }
}

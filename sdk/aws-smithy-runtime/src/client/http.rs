/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
use aws_smithy_runtime_api::client::behavior_version::BehaviorVersion;
#[cfg(feature = "default-https-client")]
use aws_smithy_runtime_api::client::http::SharedHttpClient;

/// Interceptor for connection poisoning.
pub mod connection_poisoning;

#[deprecated = "Direct HTTP test utility support from `aws-smithy-runtime` crate is deprecated. Please use the `test-util` feature from `aws-smithy-http-client` instead"]
#[cfg(feature = "test-util")]
pub mod test_util {
    #![allow(missing_docs)]

    pub use aws_smithy_http_client::test_util::{
        legacy_capture_request as capture_request, CaptureRequestHandler, CaptureRequestReceiver,
    };

    #[cfg(feature = "connector-hyper-0-14-x")]
    pub mod dvr {
        pub use aws_smithy_http_client::test_util::dvr::*;
    }

    pub use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};

    pub use aws_smithy_http_client::test_util::legacy_infallible::infallible_client_fn;

    pub use aws_smithy_http_client::test_util::NeverClient;

    #[cfg(feature = "connector-hyper-0-14-x")]
    pub use aws_smithy_http_client::test_util::NeverTcpConnector;

    #[cfg(all(feature = "connector-hyper-0-14-x", feature = "wire-mock"))]
    #[macro_use]
    pub mod wire {
        pub use aws_smithy_http_client::test_util::wire::ev;
        pub use aws_smithy_http_client::test_util::wire::match_events;
        pub use aws_smithy_http_client::test_util::wire::matcher;
        pub use aws_smithy_http_client::test_util::wire::*;
    }
}

/// Default HTTP and TLS connectors that use hyper 0.14.x and rustls.
///
/// This module is named after the hyper version number since we anticipate
/// needing to provide equivalent functionality for hyper 1.x in the future.
#[cfg(feature = "connector-hyper-0-14-x")]
#[deprecated = "hyper 0.14.x connector is deprecated, please use the `aws-smithy-http-client` crate directly instead."]
pub mod hyper_014 {
    #[allow(deprecated)]
    pub use aws_smithy_http_client::hyper_014::*;
}

/// HTTP body and body-wrapper types
pub mod body;

// NOTE: We created default client options to evolve defaults over time (e.g. allow passing a different DNS resolver)
/// Configuration options for the default HTTPS client
#[derive(Debug, Clone)]
pub(crate) struct DefaultClientOptions {
    _behavior_version: BehaviorVersion,
}

impl Default for DefaultClientOptions {
    fn default() -> Self {
        DefaultClientOptions {
            _behavior_version: BehaviorVersion::latest(),
        }
    }
}

impl DefaultClientOptions {
    /// Set the behavior version to use
    #[allow(unused)]
    pub(crate) fn with_behavior_version(mut self, behavior_version: BehaviorVersion) -> Self {
        self._behavior_version = behavior_version;
        self
    }
}

/// Creates an HTTPS client using the default TLS provider
#[cfg(feature = "default-https-client")]
pub(crate) fn default_https_client(_options: DefaultClientOptions) -> Option<SharedHttpClient> {
    use aws_smithy_http_client::{tls, Builder};
    tracing::trace!("creating a new default hyper 1.x client using rustls<aws-lc>");
    Some(
        Builder::new()
            .tls_provider(tls::Provider::Rustls(
                tls::rustls_provider::CryptoMode::AwsLc,
            ))
            .build_https(),
    )
}

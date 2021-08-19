/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */
use aws_auth::provider::AsyncProvideCredentials;
use aws_hyper::DynConnector;

pub use default_provider_chain::DefaultProviderChain;

pub mod default_provider_chain;
pub mod profile;

/// Credentials Provider that evaluates a series of providers
pub mod chain;
mod sts_util;
mod test_case;
pub mod web_identity_token;

// create a default connector given the currently enabled cargo features.
// rustls  | native tls | result
// -----------------------------
// yes     | yes        | rustls
// yes     | no         | rustls
// no      | yes        | native_tls
// no      | no         | no default

fn must_have_connector() -> DynConnector {
    default_connector().expect("A connector was not available. Either set a custom connector or enable the `rustls` and `native-tls` crate features.")
}

#[cfg(feature = "rustls")]
fn default_connector() -> Option<DynConnector> {
    Some(DynConnector::new(smithy_client::conns::https()))
}

#[cfg(all(not(feature = "rustls"), feature = "native-tls"))]
fn default_connector() -> Option<DynConnector> {
    Some(DynConnector::new(smithy_client::conns::native_tls()))
}

#[cfg(not(any(feature = "rustls", feature = "native-tls")))]
fn default_connector() -> Option<DynConnector> {
    None
}

// because this doesn't provide any configuration, a runtime and connector must be provided.
#[cfg(all(any(feature = "native-tls", feature = "rustls"), feature = "rt-tokio"))]
/// Default AWS provider chain
///
/// This provider chain will use defaults for all settings. The region will be resolved with the default
/// provider chain. To construct a custom provider, use [`default_provider_chain::Builder`](default_provider_chain::Builder).
pub fn default_provider() -> impl AsyncProvideCredentials {
    let resolved_region = aws_types::region::default_provider();
    default_provider_chain::Builder::default()
        .region(&resolved_region)
        .build()
}

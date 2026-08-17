/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
use std::fmt;

use crate::cfg::{cfg_rustls, cfg_s2n_tls};
use crate::HttpClientError;

/// Choice of underlying cryptography library
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum Provider {
    #[cfg(feature = "__rustls")]
    /// TLS provider based on [rustls](https://github.com/rustls/rustls)
    Rustls(rustls_provider::CryptoMode),
    /// TLS provider based on [s2n-tls](https://github.com/aws/s2n-tls)
    #[cfg(feature = "s2n-tls")]
    S2nTls,
}

#[cfg(not(all(aws_sdk_unstable, feature = "__rustls")))]
impl Eq for Provider {}

/// TLS related configuration object
#[derive(Debug, Clone)]
pub struct TlsContext {
    #[allow(unused)]
    trust_store: TrustStore,
    #[allow(unused)]
    additional_server_names: Vec<ServerName>,
}

impl TlsContext {
    /// Create a new [TlsContext] builder
    pub fn builder() -> TlsContextBuilder {
        TlsContextBuilder::new()
    }
}

impl Default for TlsContext {
    fn default() -> Self {
        TlsContext::builder().build().expect("valid default config")
    }
}

/// Builder for TLS related configuration
#[derive(Debug)]
pub struct TlsContextBuilder {
    trust_store: TrustStore,
    additional_server_names: Vec<ServerName>,
}

impl TlsContextBuilder {
    fn new() -> Self {
        TlsContextBuilder {
            trust_store: TrustStore::default(),
            additional_server_names: Vec::default(),
        }
    }

    /// Configure the trust store to use for the TLS context
    pub fn with_trust_store(mut self, trust_store: TrustStore) -> Self {
        self.trust_store = trust_store;
        self
    }

    /// Configure additional server names to accept during TLS certificate verification.
    pub fn with_additional_server_names(
        mut self,
        additional_server_names: Vec<ServerName>,
    ) -> Self {
        self.additional_server_names = additional_server_names;
        self
    }

    /// Build a new [TlsContext]
    pub fn build(self) -> Result<TlsContext, HttpClientError> {
        Ok(TlsContext {
            trust_store: self.trust_store,
            additional_server_names: self.additional_server_names,
        })
    }
}

/// PEM encoded certificate
#[allow(unused)]
#[derive(Debug, Clone)]
struct CertificatePEM(Vec<u8>);

impl From<&[u8]> for CertificatePEM {
    fn from(value: &[u8]) -> Self {
        CertificatePEM(value.to_vec())
    }
}

/// Container for root certificates able to provide a root-of-trust for connection authentication
///
/// Platform native root certificates are enabled by default. To start with a clean trust
/// store use [TrustStore::empty]
#[derive(Debug, Clone)]
pub struct TrustStore {
    enable_native_roots: bool,
    custom_certs: Vec<CertificatePEM>,
}

impl TrustStore {
    /// Create a new empty trust store
    pub fn empty() -> Self {
        Self {
            enable_native_roots: false,
            custom_certs: Vec::new(),
        }
    }

    /// Enable or disable using the platform's native trusted root certificate store
    ///
    /// Default: true
    pub fn with_native_roots(mut self, enable_native_roots: bool) -> Self {
        self.enable_native_roots = enable_native_roots;
        self
    }

    /// Add the PEM encoded certificate to the trust store
    ///
    /// This may be called more than once to add multiple certificates.
    /// NOTE: PEM certificate contents are not validated until passed to the configured
    /// TLS provider.
    pub fn with_pem_certificate(mut self, pem_bytes: impl Into<Vec<u8>>) -> Self {
        // ideally we'd validate here but rustls-pki-types converts to DER when loading and S2N
        // still expects PEM encoding. Store the raw bytes and let the TLS implementation validate
        self.custom_certs.push(CertificatePEM(pem_bytes.into()));
        self
    }

    /// Add the PEM encoded certificate to the trust store
    ///
    /// This may be called more than once to add multiple certificates.
    /// NOTE: PEM certificate contents are not validated until passed to the configured
    /// TLS provider.
    pub fn add_pem_certificate(&mut self, pem_bytes: impl Into<Vec<u8>>) -> &mut Self {
        self.custom_certs.push(CertificatePEM(pem_bytes.into()));
        self
    }
}

impl Default for TrustStore {
    fn default() -> Self {
        Self {
            enable_native_roots: true,
            custom_certs: Vec::new(),
        }
    }
}

/// A server name for TLS connections.
///
/// This represents a DNS hostname or IP address used for TLS Server Name
/// Indication (SNI) and certificate verification.
///
/// # Examples
///
/// ```
/// use aws_smithy_http_client::tls::ServerName;
///
/// let name = ServerName::try_from("example.com").unwrap();
/// let ip_name = ServerName::try_from("127.0.0.1").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerName(rustls_pki_types::ServerName<'static>);

/// Error returned when a server name string is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidServerName {
    name: String,
}

impl fmt::Display for InvalidServerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid server name: {:?}", self.name)
    }
}

impl std::error::Error for InvalidServerName {}

impl TryFrom<String> for ServerName {
    type Error = InvalidServerName;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        match rustls_pki_types::ServerName::try_from(name.as_str()) {
            Ok(sn) => Ok(ServerName(sn.to_owned())),
            Err(_) => Err(InvalidServerName { name }),
        }
    }
}

impl TryFrom<&str> for ServerName {
    type Error = InvalidServerName;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        match rustls_pki_types::ServerName::try_from(name) {
            Ok(sn) => Ok(ServerName(sn.to_owned())),
            Err(_) => Err(InvalidServerName {
                name: name.to_owned(),
            }),
        }
    }
}

cfg_rustls! {
    /// rustls based support and adapters
    pub mod rustls_provider;
}

cfg_s2n_tls! {
    /// s2n-tls based support and adapters
    pub(crate) mod s2n_tls_provider;
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
use crate::cfg::{cfg_rustls, cfg_s2n_tls};
use crate::HttpClientError;

/// Choice of underlying cryptography library
#[derive(Debug, Eq, PartialEq, Clone)]
#[non_exhaustive]
pub enum Provider {
    #[cfg(any(
        feature = "rustls-aws-lc",
        feature = "rustls-aws-lc-fips",
        feature = "rustls-ring"
    ))]
    /// TLS provider based on [rustls](https://github.com/rustls/rustls)
    Rustls(rustls_provider::CryptoMode),
    /// TLS provider based on [s2n-tls](https://github.com/aws/s2n-tls)
    #[cfg(feature = "s2n-tls")]
    S2nTls,
}

/// TLS related configuration object
#[derive(Debug, Clone)]
pub struct TlsContext {
    #[allow(unused)]
    trust_store: TrustStore,
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
}

impl TlsContextBuilder {
    fn new() -> Self {
        TlsContextBuilder {
            trust_store: TrustStore::default(),
        }
    }

    /// Configure the trust store to use for the TLS context
    pub fn with_trust_store(mut self, trust_store: TrustStore) -> Self {
        self.trust_store = trust_store;
        self
    }

    /// Build a new [TlsContext]
    pub fn build(self) -> Result<TlsContext, HttpClientError> {
        Ok(TlsContext {
            trust_store: self.trust_store,
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

cfg_rustls! {
    /// rustls based support and adapters
    pub mod rustls_provider {
        use crate::client::tls::Provider;
        use rustls::crypto::CryptoProvider;

        /// Choice of underlying cryptography library (this only applies to rustls)
        #[derive(Debug, Eq, PartialEq, Clone)]
        #[non_exhaustive]
        pub enum CryptoMode {
            /// Crypto based on [ring](https://github.com/briansmith/ring)
            #[cfg(feature = "rustls-ring")]
            Ring,
            /// Crypto based on [aws-lc](https://github.com/aws/aws-lc-rs)
            #[cfg(feature = "rustls-aws-lc")]
            AwsLc,
            /// FIPS compliant variant of [aws-lc](https://github.com/aws/aws-lc-rs)
            #[cfg(feature = "rustls-aws-lc-fips")]
            AwsLcFips,
        }

        impl CryptoMode {
            fn provider(self) -> CryptoProvider {
                match self {
                    #[cfg(feature = "rustls-aws-lc")]
                    CryptoMode::AwsLc => rustls::crypto::aws_lc_rs::default_provider(),

                    #[cfg(feature = "rustls-ring")]
                    CryptoMode::Ring => rustls::crypto::ring::default_provider(),

                    #[cfg(feature = "rustls-aws-lc-fips")]
                    CryptoMode::AwsLcFips => {
                        let provider = rustls::crypto::default_fips_provider();
                        assert!(
                            provider.fips(),
                            "FIPS was requested but the provider did not support FIPS"
                        );
                        provider
                    }
                }
            }
        }

        impl Provider {
            /// Create a TLS provider based on [rustls](https://github.com/rustls/rustls)
            /// and the given [`CryptoMode`]
            pub fn rustls(mode: CryptoMode) -> Provider {
                Provider::Rustls(mode)
            }
        }

        pub(crate) mod build_connector {
            use crate::client::tls::rustls_provider::CryptoMode;
            use crate::tls::TlsContext;
            use hyper_util::client::legacy as client;
            use client::connect::HttpConnector;
            use rustls::crypto::CryptoProvider;
            use std::sync::Arc;
            use rustls_pki_types::CertificateDer;
            use rustls_pki_types::pem::PemObject;
            use rustls_native_certs::CertificateResult;
            use std::sync::LazyLock;

            /// Cached native certificates
            ///
            /// Creating a `with_native_roots()` hyper_rustls client re-loads system certs
            /// each invocation (which can take 300ms on OSx). Cache the loaded certs
            /// to avoid repeatedly incurring that cost.
            pub(crate) static NATIVE_ROOTS: LazyLock<Vec<CertificateDer<'static>>> = LazyLock::new(|| {
                let CertificateResult { certs, errors, .. } = rustls_native_certs::load_native_certs();
                if !errors.is_empty() {
                    tracing::warn!("native root CA certificate loading errors: {errors:?}")
                }

                if certs.is_empty() {
                    tracing::warn!("no native root CA certificates found!");
                }

                // NOTE: unlike hyper-rustls::with_native_roots we don't validate here, we'll do that later
                // for now we have a collection of certs that may or may not be valid.
                certs
            });

            fn restrict_ciphers(base: CryptoProvider) -> CryptoProvider {
                let suites = &[
                    rustls::CipherSuite::TLS13_AES_256_GCM_SHA384,
                    rustls::CipherSuite::TLS13_AES_128_GCM_SHA256,
                    // TLS1.2 suites
                    rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
                    rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                    rustls::CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                    rustls::CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
                    rustls::CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
                ];
                let supported_suites = suites
                    .iter()
                    .flat_map(|suite| {
                        base.cipher_suites
                            .iter()
                            .find(|s| &s.suite() == suite)
                            .cloned()
                    })
                    .collect::<Vec<_>>();
                CryptoProvider {
                    cipher_suites: supported_suites,
                    ..base
                }
            }

            impl TlsContext {
                fn rustls_root_certs(&self) -> rustls::RootCertStore {
                    let mut roots = rustls::RootCertStore::empty();
                    if self.trust_store.enable_native_roots {
                        let (valid, _invalid) = roots.add_parsable_certificates(
                           NATIVE_ROOTS.clone()
                        );
                        debug_assert!(valid > 0, "TrustStore configured to enable native roots but no valid root certificates parsed!");
                    }

                    for pem_cert in &self.trust_store.custom_certs {
                        let ders = CertificateDer::pem_slice_iter(&pem_cert.0).collect::<Result<Vec<_>, _> >().expect("valid PEM certificate");
                        for cert in ders {
                            roots.add(cert).expect("cert parsable")
                        }
                    }

                    roots
                }
            }

            pub(crate) fn wrap_connector<R>(
                mut conn: HttpConnector<R>,
                crypto_mode: CryptoMode,
                tls_context: &TlsContext,
            ) -> hyper_rustls::HttpsConnector<HttpConnector<R>> {
                let root_certs = tls_context.rustls_root_certs();
                conn.enforce_http(false);
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_tls_config(
                        rustls::ClientConfig::builder_with_provider(Arc::new(restrict_ciphers(crypto_mode.provider())))
                            .with_safe_default_protocol_versions()
                            .expect("Error with the TLS configuration. Please file a bug report under https://github.com/smithy-lang/smithy-rs/issues.")
                            .with_root_certificates(root_certs)
                            .with_no_client_auth()
                    )
                    .https_or_http()
                    .enable_http1()
                    .enable_http2()
                    .wrap_connector(conn)
            }
        }
    }
}

cfg_s2n_tls! {
    /// s2n-tls based support and adapters
    pub(crate) mod s2n_tls_provider {
        pub(crate) mod build_connector {
            use hyper_util::client::legacy as client;
            use client::connect::HttpConnector;
            use s2n_tls::security::Policy;
            use crate::tls::TlsContext;
            use std::sync::LazyLock;

            // Default S2N security policy which sets protocol versions and cipher suites
            //  See https://aws.github.io/s2n-tls/usage-guide/ch06-security-policies.html
            const S2N_POLICY_VERSION: &str = "20230317";

            fn base_config() -> s2n_tls::config::Builder {
                let mut builder = s2n_tls::config::Config::builder();
                let policy = Policy::from_version(S2N_POLICY_VERSION).unwrap();
                builder.set_security_policy(&policy).expect("valid s2n security policy");
                // default is true
                builder.with_system_certs(false).unwrap();
                builder
            }

            static CACHED_CONFIG: LazyLock<s2n_tls::config::Config> = LazyLock::new(|| {
                let mut config = base_config();
                config.with_system_certs(true).unwrap();
                // actually loads the system certs
                config.build().expect("valid s2n config")
            });

            impl TlsContext {
                fn s2n_config(&self) -> s2n_tls::config::Config {
                    // TODO(s2n-tls): s2n does not support turning a config back into a builder or a way to load a trust store and re-use it
                    // instead if we are only using the defaults then use a cached config, otherwise pay the cost to build a new one
                    if self.trust_store.enable_native_roots && self.trust_store.custom_certs.is_empty() {
                        CACHED_CONFIG.clone()
                    } else {
                        let mut config = base_config();
                        config.with_system_certs(self.trust_store.enable_native_roots).unwrap();
                        for pem_cert in &self.trust_store.custom_certs {
                            config.trust_pem(pem_cert.0.as_slice()).expect("valid certificate");
                        }
                        config.build().expect("valid s2n config")
                    }
                }
            }

            pub(crate) fn wrap_connector<R>(
                mut http_connector: HttpConnector<R>,
                tls_context: &TlsContext,
            ) -> s2n_tls_hyper::connector::HttpsConnector<HttpConnector<R>> {
                let config = tls_context.s2n_config();
                http_connector.enforce_http(false);
                let mut builder = s2n_tls_hyper::connector::HttpsConnector::builder_with_http(http_connector, config);
                builder.with_plaintext_http(true);
                builder.build()
            }
        }
    }
}

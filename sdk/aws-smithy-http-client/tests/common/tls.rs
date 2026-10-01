/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! TLS identity and configuration shared by integration tests.

use aws_smithy_http_client::tls::{TlsContext, TlsContextBuilder, TrustStore};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::fs;
use std::io;
use std::sync::Arc;
use tokio_rustls::{rustls, rustls::ServerConfig, TlsAcceptor};

/// PEM-backed identity used by integration-test TLS servers.
///
/// The certificate contains `localhost` and `sdktest.com` as subject
/// alternative names.
pub(crate) const SERVER_IDENTITY: TestTlsIdentity =
    TestTlsIdentity::from_pem("tests/server.pem", "tests/server.rsa");

/// Certificate and private-key material for one test TLS identity.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TestTlsIdentity {
    certificate_path: &'static str,
    private_key_path: &'static str,
}

impl TestTlsIdentity {
    const fn from_pem(certificate_path: &'static str, private_key_path: &'static str) -> Self {
        Self {
            certificate_path,
            private_key_path,
        }
    }

    /// Builds a server acceptor using this identity and ALPN protocol list.
    pub(crate) fn acceptor(&self, alpn_protocols: &[&[u8]]) -> io::Result<TlsAcceptor> {
        // The test server uses one process-wide rustls crypto provider.
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        let certs = load_certs(self.certificate_path)?;
        let key = load_private_key(self.private_key_path)?;
        let mut server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|err| error(err.to_string()))?;
        server_config.alpn_protocols = alpn_protocols
            .iter()
            .map(|protocol| protocol.to_vec())
            .collect();

        Ok(TlsAcceptor::from(Arc::new(server_config)))
    }

    /// Returns a client TLS context builder that trusts this identity.
    pub(crate) fn client_context_builder(&self) -> TlsContextBuilder {
        TlsContext::builder().with_trust_store(self.trust_store())
    }

    /// Builds a client TLS context that trusts this identity.
    pub(crate) fn client_context(&self) -> TlsContext {
        self.client_context_builder()
            .build()
            .expect("test TLS identity produces a valid client context")
    }

    fn trust_store(&self) -> TrustStore {
        let pem_contents =
            fs::read(self.certificate_path).expect("failed to read test TLS identity certificate");
        TrustStore::empty().with_pem_certificate(pem_contents)
    }
}

fn error(err: String) -> io::Error {
    io::Error::other(err)
}

fn load_certs(filename: &str) -> io::Result<Vec<CertificateDer<'static>>> {
    let certfile = fs::File::open(filename)
        .map_err(|err| error(format!("failed to open {filename}: {err}")))?;
    let mut reader = io::BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader).collect()
}

fn load_private_key(filename: &str) -> io::Result<PrivateKeyDer<'static>> {
    let keyfile = fs::File::open(filename)
        .map_err(|err| error(format!("failed to open {filename}: {err}")))?;
    let mut reader = io::BufReader::new(keyfile);

    rustls_pemfile::private_key(&mut reader)
        .map(|key| key.expect("no private key found in PEM file"))
}

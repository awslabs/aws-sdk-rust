/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! TLS setup shared by integration-test servers and clients.

use aws_smithy_http_client::tls::{TlsContext, TrustStore};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::fs;
use std::io;
use std::sync::Arc;
use tokio_rustls::{rustls, rustls::ServerConfig, TlsAcceptor};

const SERVER_CERT_PATH: &str = "tests/server.pem";
const SERVER_KEY_PATH: &str = "tests/server.rsa";

pub(crate) fn server_tls_acceptor(alpn_protocols: &[&[u8]]) -> io::Result<TlsAcceptor> {
    // Set the process-wide crypto provider used by the test server.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let certs = load_certs(SERVER_CERT_PATH)?;
    let key = load_private_key(SERVER_KEY_PATH)?;
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

pub(crate) fn server_tls_context() -> TlsContext {
    TlsContext::builder()
        .with_trust_store(server_trust_store())
        .build()
        .expect("failed to build TlsContext with test server certificate")
}

/// A [`TrustStore`] containing only the test server certificate.
pub(crate) fn server_trust_store() -> TrustStore {
    let pem_contents =
        fs::read(SERVER_CERT_PATH).expect("failed to read server certificate for test TLS context");
    TrustStore::empty().with_pem_certificate(pem_contents)
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

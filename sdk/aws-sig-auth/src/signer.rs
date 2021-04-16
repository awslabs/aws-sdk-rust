/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_auth::Credentials;
use aws_types::region::SigningRegion;
use aws_types::SigningService;
use std::error::Error;
use std::time::SystemTime;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum SigningAlgorithm {
    SigV4,
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum HttpSignatureType {
    /// A signature for a full http request should be computed, with header updates applied to the signing result.
    HttpRequestHeaders,
    /* Currently Unsupported
    /// A signature for a full http request should be computed, with query param updates applied to the signing result.
    ///
    /// This is typically used for presigned URLs & is currently unsupported.
    HttpRequestQueryParams,
     */
}

/// Signing Configuration for an Operation
///
/// Although these fields MAY be customized on a per request basis, they are generally static
/// for a given operation
#[derive(Clone, PartialEq, Eq)]
pub struct OperationSigningConfig {
    pub algorithm: SigningAlgorithm,
    pub signature_type: HttpSignatureType,
    pub signing_options: SigningOptions,
}

impl OperationSigningConfig {
    /// Placeholder method to provide a the signing configuration used for most operation
    ///
    /// In the future, we will code-generate a default configuration for each service
    pub fn default_config() -> Self {
        OperationSigningConfig {
            algorithm: SigningAlgorithm::SigV4,
            signature_type: HttpSignatureType::HttpRequestHeaders,
            signing_options: SigningOptions { _private: () },
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct SigningOptions {
    _private: (),
    /*
    Currently unsupported:
    pub double_uri_encode: bool,
    pub normalize_uri_path: bool,
    pub omit_session_token: bool,
     */
}

/// Signing Configuration for an individual Request
///
/// These fields may vary on a per-request basis
#[derive(Clone, PartialEq, Eq)]
pub struct RequestConfig<'a> {
    pub request_ts: SystemTime,
    pub region: &'a SigningRegion,
    pub service: &'a SigningService,
}

#[derive(Clone, Default)]
pub struct SigV4Signer {
    // In the future, the SigV4Signer will use the CRT signer. This will require constructing
    // and holding an instance of the signer, so prevent people from constructing a SigV4Signer without
    // going through the constructor.
    _private: (),
}

pub type SigningError = Box<dyn Error + Send + Sync>;

impl SigV4Signer {
    pub fn new() -> Self {
        SigV4Signer { _private: () }
    }

    /// Sign a request using the SigV4 Protocol
    ///
    /// Although the direct signing implementation MAY be used directly. End users will not typically
    /// interact with this code. It is generally used via middleware in the request pipeline. See [`SigV4SigningStage`](crate::middleware::SigV4SigningStage).
    pub fn sign<B>(
        &self,
        // There is currently only 1 way to sign, so operation level configuration is unused
        _operation_config: &OperationSigningConfig,
        request_config: &RequestConfig<'_>,
        credentials: &Credentials,
        request: &mut http::Request<B>,
    ) -> Result<(), SigningError>
    where
        B: AsRef<[u8]>,
    {
        let sigv4_creds = aws_sigv4_poc::Credentials {
            access_key: credentials.access_key_id().to_string(),
            secret_key: credentials.secret_access_key().to_string(),
            security_token: credentials.session_token().map(|s| s.to_string()),
        };
        let date = request_config.request_ts;
        for (key, value) in aws_sigv4_poc::sign_core(
            request,
            &sigv4_creds,
            request_config.region.as_ref(),
            request_config.service.as_ref(),
            date,
        ) {
            request
                .headers_mut()
                .append(key.header_name(), value.parse()?);
        }

        Ok(())
    }
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_auth::Credentials;
use aws_sigv4::http_request::{
    calculate_signing_headers, PayloadChecksumKind, SigningSettings, UriEncoding,
};
use aws_types::region::SigningRegion;
use aws_types::SigningService;
use http::header::HeaderName;
use smithy_http::body::SdkBody;
use std::error::Error;
use std::fmt;
use std::time::SystemTime;

use crate::middleware::Signature;
pub use aws_sigv4::http_request::SignableBody;

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
#[non_exhaustive]
pub struct OperationSigningConfig {
    pub algorithm: SigningAlgorithm,
    pub signature_type: HttpSignatureType,
    pub signing_options: SigningOptions,
    pub signing_requirements: SigningRequirements,
}

impl OperationSigningConfig {
    /// Placeholder method to provide a the signing configuration used for most operation
    ///
    /// In the future, we will code-generate a default configuration for each service
    pub fn default_config() -> Self {
        OperationSigningConfig {
            algorithm: SigningAlgorithm::SigV4,
            signature_type: HttpSignatureType::HttpRequestHeaders,
            signing_options: SigningOptions {
                double_uri_encode: true,
                content_sha256_header: false,
            },
            signing_requirements: SigningRequirements::Required,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SigningRequirements {
    /// A signature MAY be added if credentials are defined
    Optional,

    /// A signature MUST be added.
    ///
    /// If no credentials are provided, this will return an error without dispatching the operation.
    Required,

    /// A signature MUST NOT be added.
    Disabled,
}

#[derive(Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct SigningOptions {
    pub double_uri_encode: bool,
    pub content_sha256_header: bool,
    /*
    Currently unsupported:
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
    pub payload_override: Option<&'a SignableBody<'static>>,
}

#[derive(Clone, Default)]
pub struct SigV4Signer {
    // In the future, the SigV4Signer will use the CRT signer. This will require constructing
    // and holding an instance of the signer, so prevent people from constructing a SigV4Signer without
    // going through the constructor.
    _private: (),
}

impl fmt::Debug for SigV4Signer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut formatter = f.debug_struct("SigV4Signer");
        formatter.finish()
    }
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
    pub fn sign(
        &self,
        operation_config: &OperationSigningConfig,
        request_config: &RequestConfig<'_>,
        credentials: &Credentials,
        request: &mut http::Request<SdkBody>,
    ) -> Result<Signature, SigningError> {
        let mut settings = SigningSettings::default();
        settings.uri_encoding = if operation_config.signing_options.double_uri_encode {
            UriEncoding::Double
        } else {
            UriEncoding::Single
        };
        settings.payload_checksum_kind = if operation_config.signing_options.content_sha256_header {
            PayloadChecksumKind::XAmzSha256
        } else {
            PayloadChecksumKind::NoHeader
        };
        let sigv4_config = aws_sigv4::http_request::SigningParams {
            access_key: credentials.access_key_id(),
            secret_key: credentials.secret_access_key(),
            security_token: credentials.session_token(),
            region: request_config.region.as_ref(),
            service_name: request_config.service.as_ref(),
            date_time: request_config.request_ts.into(),
            settings,
        };

        // A body that is already in memory can be signed directly. A  body that is not in memory
        // (any sort of streaming body) will be signed via UNSIGNED-PAYLOAD.
        let signable_body = request_config
            .payload_override
            // the payload_override is a cheap clone because it contains either a
            // reference or a short checksum (we're not cloning the entire body)
            .cloned()
            .unwrap_or_else(|| {
                request
                    .body()
                    .bytes()
                    .map(SignableBody::Bytes)
                    .unwrap_or(SignableBody::UnsignedPayload)
            });

        let (signing_headers, signature) =
            calculate_signing_headers(request, signable_body, &sigv4_config)?.into_parts();
        for (key, value) in signing_headers {
            request
                .headers_mut()
                .append(HeaderName::from_static(key), value);
        }

        Ok(Signature::new(signature))
    }
}

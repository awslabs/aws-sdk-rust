/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::Credentials;
use aws_sigv4::http_request::{
    sign, PayloadChecksumKind, PercentEncodingMode, SessionTokenMode, SignableRequest,
    SignatureLocation, SigningParams, SigningSettings, UriPathNormalizationMode,
};
use aws_smithy_http::body::SdkBody;
use aws_types::region::SigningRegion;
use aws_types::SigningService;
use std::fmt;
use std::time::{Duration, SystemTime};

use crate::middleware::Signature;
pub use aws_sigv4::http_request::SignableBody;
pub type SigningError = aws_sigv4::http_request::SigningError;

const EXPIRATION_WARNING: &str = "Presigned request will expire before the given \
    `expires_in` duration because the credentials used to sign it will expire first.";

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum SigningAlgorithm {
    SigV4,
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum HttpSignatureType {
    /// A signature for a full http request should be computed, with header updates applied to the signing result.
    HttpRequestHeaders,

    /// A signature for a full http request should be computed, with query param updates applied to the signing result.
    ///
    /// This is typically used for presigned URLs.
    HttpRequestQueryParams,
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
    pub expires_in: Option<Duration>,
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
                normalize_uri_path: true,
                omit_session_token: false,
            },
            signing_requirements: SigningRequirements::Required,
            expires_in: None,
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
    pub normalize_uri_path: bool,
    pub omit_session_token: bool,
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

impl SigV4Signer {
    pub fn new() -> Self {
        SigV4Signer { _private: () }
    }

    fn settings(operation_config: &OperationSigningConfig) -> SigningSettings {
        let mut settings = SigningSettings::default();
        settings.percent_encoding_mode = if operation_config.signing_options.double_uri_encode {
            PercentEncodingMode::Double
        } else {
            PercentEncodingMode::Single
        };
        settings.payload_checksum_kind = if operation_config.signing_options.content_sha256_header {
            PayloadChecksumKind::XAmzSha256
        } else {
            PayloadChecksumKind::NoHeader
        };
        settings.uri_path_normalization_mode =
            if operation_config.signing_options.normalize_uri_path {
                UriPathNormalizationMode::Enabled
            } else {
                UriPathNormalizationMode::Disabled
            };
        settings.session_token_mode = if operation_config.signing_options.omit_session_token {
            SessionTokenMode::Exclude
        } else {
            SessionTokenMode::Include
        };
        settings.signature_location = match operation_config.signature_type {
            HttpSignatureType::HttpRequestHeaders => SignatureLocation::Headers,
            HttpSignatureType::HttpRequestQueryParams => SignatureLocation::QueryParams,
        };
        settings.expires_in = operation_config.expires_in;
        settings
    }

    fn signing_params<'a>(
        settings: SigningSettings,
        credentials: &'a Credentials,
        request_config: &'a RequestConfig<'a>,
    ) -> SigningParams<'a> {
        if let Some(expires_in) = settings.expires_in {
            if let Some(creds_expires_time) = credentials.expiry() {
                let presigned_expires_time = request_config.request_ts + expires_in;
                if presigned_expires_time > creds_expires_time {
                    tracing::warn!(EXPIRATION_WARNING);
                }
            }
        }

        let mut builder = SigningParams::builder()
            .access_key(credentials.access_key_id())
            .secret_key(credentials.secret_access_key())
            .region(request_config.region.as_ref())
            .service_name(request_config.service.as_ref())
            .time(request_config.request_ts)
            .settings(settings);
        builder.set_security_token(credentials.session_token());
        builder.build().expect("all required fields set")
    }

    /// Sign a request using the SigV4 Protocol
    ///
    /// Although this function may be used, end users will not typically
    /// interact with this code. It is generally used via middleware in the request pipeline. See [`SigV4SigningStage`](crate::middleware::SigV4SigningStage).
    pub fn sign(
        &self,
        operation_config: &OperationSigningConfig,
        request_config: &RequestConfig<'_>,
        credentials: &Credentials,
        request: &mut http::Request<SdkBody>,
    ) -> Result<Signature, SigningError> {
        let settings = Self::settings(operation_config);
        let signing_params = Self::signing_params(settings, credentials, request_config);

        let (signing_instructions, signature) = {
            // A body that is already in memory can be signed directly. A body that is not in memory
            // (any sort of streaming body or presigned request) will be signed via UNSIGNED-PAYLOAD.
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

            let signable_request = SignableRequest::new(
                request.method(),
                request.uri(),
                request.headers(),
                signable_body,
            );
            sign(signable_request, &signing_params)?
        }
        .into_parts();

        signing_instructions.apply_to_request(request);

        Ok(Signature::new(signature))
    }
}

#[cfg(test)]
mod tests {
    use super::{RequestConfig, SigV4Signer, EXPIRATION_WARNING};
    use aws_credential_types::Credentials;
    use aws_sigv4::http_request::SigningSettings;
    use aws_types::region::SigningRegion;
    use aws_types::SigningService;
    use std::time::{Duration, SystemTime};
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn expiration_warning() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        let creds_expire_in = Duration::from_secs(100);

        let mut settings = SigningSettings::default();
        settings.expires_in = Some(creds_expire_in - Duration::from_secs(10));

        let credentials = Credentials::new(
            "test-access-key",
            "test-secret-key",
            Some("test-session-token".into()),
            Some(now + creds_expire_in),
            "test",
        );
        let request_config = RequestConfig {
            request_ts: now,
            region: &SigningRegion::from_static("test"),
            service: &SigningService::from_static("test"),
            payload_override: None,
        };
        SigV4Signer::signing_params(settings, &credentials, &request_config);
        assert!(!logs_contain(EXPIRATION_WARNING));

        let mut settings = SigningSettings::default();
        settings.expires_in = Some(creds_expire_in + Duration::from_secs(10));

        SigV4Signer::signing_params(settings, &credentials, &request_config);
        assert!(logs_contain(EXPIRATION_WARNING));
    }
}

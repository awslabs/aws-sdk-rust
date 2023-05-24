/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Auth implementations for SigV4.
pub mod sigv4 {
    use aws_credential_types::Credentials;
    use aws_sigv4::http_request::{
        sign, PayloadChecksumKind, PercentEncodingMode, SessionTokenMode, SignableBody,
        SignableRequest, SignatureLocation, SigningParams, SigningSettings,
        UriPathNormalizationMode,
    };
    use aws_smithy_http::property_bag::PropertyBag;
    use aws_smithy_runtime_api::client::identity::{Identity, IdentityResolver, IdentityResolvers};
    use aws_smithy_runtime_api::client::orchestrator::{
        BoxError, HttpAuthScheme, HttpRequest, HttpRequestSigner,
    };
    use aws_types::region::SigningRegion;
    use aws_types::SigningService;
    use std::time::{Duration, SystemTime};

    const EXPIRATION_WARNING: &str = "Presigned request will expire before the given \
        `expires_in` duration because the credentials used to sign it will expire first.";

    /// Auth scheme ID for SigV4.
    pub const SCHEME_ID: &str = "sigv4";

    /// SigV4 auth scheme.
    #[derive(Debug, Default)]
    pub struct SigV4HttpAuthScheme {
        signer: SigV4HttpRequestSigner,
    }

    impl SigV4HttpAuthScheme {
        /// Creates a new `SigV4HttpAuthScheme`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl HttpAuthScheme for SigV4HttpAuthScheme {
        fn scheme_id(&self) -> &'static str {
            SCHEME_ID
        }

        fn identity_resolver<'a>(
            &self,
            identity_resolvers: &'a IdentityResolvers,
        ) -> Option<&'a dyn IdentityResolver> {
            identity_resolvers.identity_resolver(self.scheme_id())
        }

        fn request_signer(&self) -> &dyn HttpRequestSigner {
            &self.signer
        }
    }

    /// Type of SigV4 signature.
    #[derive(Debug, Eq, PartialEq, Clone, Copy)]
    pub enum HttpSignatureType {
        /// A signature for a full http request should be computed, with header updates applied to the signing result.
        HttpRequestHeaders,

        /// A signature for a full http request should be computed, with query param updates applied to the signing result.
        ///
        /// This is typically used for presigned URLs.
        HttpRequestQueryParams,
    }

    /// Signing options for SigV4.
    #[derive(Clone, Debug, Eq, PartialEq)]
    #[non_exhaustive]
    pub struct SigningOptions {
        /// Apply URI encoding twice.
        pub double_uri_encode: bool,
        /// Apply a SHA-256 payload checksum.
        pub content_sha256_header: bool,
        /// Normalize the URI path before signing.
        pub normalize_uri_path: bool,
        /// Omit the session token from the signature.
        pub omit_session_token: bool,
        /// Optional override for the payload to be used in signing.
        pub payload_override: Option<SignableBody<'static>>,
        /// Signature type.
        pub signature_type: HttpSignatureType,
        /// Whether or not the signature is optional.
        pub signing_optional: bool,
        /// Optional expiration (for presigning)
        pub expires_in: Option<Duration>,
        /// Timestamp to sign with.
        pub request_timestamp: SystemTime,
    }

    impl Default for SigningOptions {
        fn default() -> Self {
            Self {
                double_uri_encode: true,
                content_sha256_header: false,
                normalize_uri_path: true,
                omit_session_token: false,
                payload_override: None,
                signature_type: HttpSignatureType::HttpRequestHeaders,
                signing_optional: false,
                expires_in: None,
                request_timestamp: SystemTime::now(),
            }
        }
    }

    /// SigV4 signing configuration for an operation
    ///
    /// Although these fields MAY be customized on a per request basis, they are generally static
    /// for a given operation
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct SigV4OperationSigningConfig {
        /// AWS Region to sign for.
        pub region: SigningRegion,
        /// AWS Service to sign for.
        pub service: SigningService,
        /// Signing options.
        pub signing_options: SigningOptions,
    }

    /// SigV4 HTTP request signer.
    #[derive(Debug, Default)]
    pub struct SigV4HttpRequestSigner;

    impl SigV4HttpRequestSigner {
        /// Creates a new signer instance.
        pub fn new() -> Self {
            Self
        }

        fn settings(operation_config: &SigV4OperationSigningConfig) -> SigningSettings {
            let mut settings = SigningSettings::default();
            settings.percent_encoding_mode = if operation_config.signing_options.double_uri_encode {
                PercentEncodingMode::Double
            } else {
                PercentEncodingMode::Single
            };
            settings.payload_checksum_kind =
                if operation_config.signing_options.content_sha256_header {
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
            settings.signature_location = match operation_config.signing_options.signature_type {
                HttpSignatureType::HttpRequestHeaders => SignatureLocation::Headers,
                HttpSignatureType::HttpRequestQueryParams => SignatureLocation::QueryParams,
            };
            settings.expires_in = operation_config.signing_options.expires_in;
            settings
        }

        fn signing_params<'a>(
            settings: SigningSettings,
            credentials: &'a Credentials,
            operation_config: &'a SigV4OperationSigningConfig,
        ) -> SigningParams<'a> {
            if let Some(expires_in) = settings.expires_in {
                if let Some(creds_expires_time) = credentials.expiry() {
                    let presigned_expires_time =
                        operation_config.signing_options.request_timestamp + expires_in;
                    if presigned_expires_time > creds_expires_time {
                        tracing::warn!(EXPIRATION_WARNING);
                    }
                }
            }

            let mut builder = SigningParams::builder()
                .access_key(credentials.access_key_id())
                .secret_key(credentials.secret_access_key())
                .region(operation_config.region.as_ref())
                .service_name(operation_config.service.as_ref())
                .time(operation_config.signing_options.request_timestamp)
                .settings(settings);
            builder.set_security_token(credentials.session_token());
            builder.build().expect("all required fields set")
        }
    }

    impl HttpRequestSigner for SigV4HttpRequestSigner {
        fn sign_request(
            &self,
            request: &mut HttpRequest,
            identity: &Identity,
            // TODO(enableNewSmithyRuntime): should this be the config bag?
            signing_properties: &PropertyBag,
        ) -> Result<(), BoxError> {
            let operation_config = signing_properties
                .get::<SigV4OperationSigningConfig>()
                .ok_or("missing operation signing config for SigV4")?;

            let credentials = if let Some(creds) = identity.data::<Credentials>() {
                creds
            } else if operation_config.signing_options.signing_optional {
                tracing::debug!("skipped SigV4 signing since signing is optional for this operation and there are no credentials");
                return Ok(());
            } else {
                return Err(format!("wrong identity type for SigV4: {identity:?}").into());
            };

            let settings = Self::settings(operation_config);
            let signing_params = Self::signing_params(settings, credentials, operation_config);

            let (signing_instructions, _signature) = {
                // A body that is already in memory can be signed directly. A body that is not in memory
                // (any sort of streaming body or presigned request) will be signed via UNSIGNED-PAYLOAD.
                let signable_body = operation_config
                    .signing_options
                    .payload_override
                    .as_ref()
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
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use aws_credential_types::Credentials;
        use aws_sigv4::http_request::SigningSettings;
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
            let operation_config = SigV4OperationSigningConfig {
                region: SigningRegion::from_static("test"),
                service: SigningService::from_static("test"),
                signing_options: SigningOptions {
                    double_uri_encode: true,
                    content_sha256_header: true,
                    normalize_uri_path: true,
                    omit_session_token: true,
                    signature_type: HttpSignatureType::HttpRequestHeaders,
                    signing_optional: false,
                    expires_in: None,
                    request_timestamp: now,
                    payload_override: None,
                },
            };
            SigV4HttpRequestSigner::signing_params(settings, &credentials, &operation_config);
            assert!(!logs_contain(EXPIRATION_WARNING));

            let mut settings = SigningSettings::default();
            settings.expires_in = Some(creds_expire_in + Duration::from_secs(10));

            SigV4HttpRequestSigner::signing_params(settings, &credentials, &operation_config);
            assert!(logs_contain(EXPIRATION_WARNING));
        }
    }
}

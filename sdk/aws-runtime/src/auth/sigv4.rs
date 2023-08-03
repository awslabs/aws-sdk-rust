/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::Credentials;
use aws_sigv4::http_request::{
    sign, PayloadChecksumKind, PercentEncodingMode, SessionTokenMode, SignableBody,
    SignableRequest, SignatureLocation, SigningParams, SigningSettings, UriPathNormalizationMode,
};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::auth::{
    AuthScheme, AuthSchemeEndpointConfig, AuthSchemeId, Signer,
};
use aws_smithy_runtime_api::client::identity::{Identity, SharedIdentityResolver};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use aws_smithy_runtime_api::client::runtime_components::{GetIdentityResolver, RuntimeComponents};
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreReplace};
use aws_smithy_types::Document;
use aws_types::region::{Region, SigningRegion};
use aws_types::SigningService;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt;
use std::time::{Duration, SystemTime};

const EXPIRATION_WARNING: &str = "Presigned request will expire before the given \
        `expires_in` duration because the credentials used to sign it will expire first.";

/// Auth scheme ID for SigV4.
pub const SCHEME_ID: AuthSchemeId = AuthSchemeId::new("sigv4");

struct EndpointAuthSchemeConfig {
    signing_region_override: Option<SigningRegion>,
    signing_service_override: Option<SigningService>,
}

#[derive(Debug)]
enum SigV4SigningError {
    MissingOperationSigningConfig,
    MissingSigningRegion,
    MissingSigningService,
    WrongIdentityType(Identity),
    BadTypeInEndpointAuthSchemeConfig(&'static str),
}

impl fmt::Display for SigV4SigningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SigV4SigningError::*;
        let mut w = |s| f.write_str(s);
        match self {
            MissingOperationSigningConfig => w("missing operation signing config for SigV4"),
            MissingSigningRegion => w("missing signing region for SigV4 signing"),
            MissingSigningService => w("missing signing service for SigV4 signing"),
            WrongIdentityType(identity) => {
                write!(f, "wrong identity type for SigV4: {identity:?}")
            }
            BadTypeInEndpointAuthSchemeConfig(field_name) => {
                write!(
                    f,
                    "unexpected type for `{field_name}` in endpoint auth scheme config",
                )
            }
        }
    }
}

impl StdError for SigV4SigningError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::MissingOperationSigningConfig => None,
            Self::MissingSigningRegion => None,
            Self::MissingSigningService => None,
            Self::WrongIdentityType(_) => None,
            Self::BadTypeInEndpointAuthSchemeConfig(_) => None,
        }
    }
}

/// SigV4 auth scheme.
#[derive(Debug, Default)]
pub struct SigV4AuthScheme {
    signer: SigV4Signer,
}

impl SigV4AuthScheme {
    /// Creates a new `SigV4AuthScheme`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl AuthScheme for SigV4AuthScheme {
    fn scheme_id(&self) -> AuthSchemeId {
        SCHEME_ID
    }

    fn identity_resolver(
        &self,
        identity_resolvers: &dyn GetIdentityResolver,
    ) -> Option<SharedIdentityResolver> {
        identity_resolvers.identity_resolver(self.scheme_id())
    }

    fn signer(&self) -> &dyn Signer {
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
    pub region: Option<SigningRegion>,
    /// AWS Service to sign for.
    pub service: Option<SigningService>,
    /// Signing options.
    pub signing_options: SigningOptions,
}

impl Storable for SigV4OperationSigningConfig {
    type Storer = StoreReplace<Self>;
}

/// SigV4 signer.
#[derive(Debug, Default)]
pub struct SigV4Signer;

impl SigV4Signer {
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
        request_timestamp: SystemTime,
    ) -> Result<SigningParams<'a>, SigV4SigningError> {
        if let Some(expires_in) = settings.expires_in {
            if let Some(creds_expires_time) = credentials.expiry() {
                let presigned_expires_time = request_timestamp + expires_in;
                if presigned_expires_time > creds_expires_time {
                    tracing::warn!(EXPIRATION_WARNING);
                }
            }
        }

        let mut builder = SigningParams::builder()
            .access_key(credentials.access_key_id())
            .secret_key(credentials.secret_access_key())
            .region(
                operation_config
                    .region
                    .as_ref()
                    .ok_or(SigV4SigningError::MissingSigningRegion)?
                    .as_ref(),
            )
            .service_name(
                operation_config
                    .service
                    .as_ref()
                    .ok_or(SigV4SigningError::MissingSigningService)?
                    .as_ref(),
            )
            .time(request_timestamp)
            .settings(settings);
        builder.set_security_token(credentials.session_token());
        Ok(builder.build().expect("all required fields set"))
    }

    fn extract_operation_config<'a>(
        auth_scheme_endpoint_config: AuthSchemeEndpointConfig<'a>,
        config_bag: &'a ConfigBag,
    ) -> Result<Cow<'a, SigV4OperationSigningConfig>, SigV4SigningError> {
        let operation_config = config_bag
            .load::<SigV4OperationSigningConfig>()
            .ok_or(SigV4SigningError::MissingOperationSigningConfig)?;

        let signing_region = config_bag.load::<SigningRegion>();
        let signing_service = config_bag.load::<SigningService>();

        let EndpointAuthSchemeConfig {
            signing_region_override,
            signing_service_override,
        } = Self::extract_endpoint_auth_scheme_config(auth_scheme_endpoint_config)?;

        match (
            signing_region_override.or_else(|| signing_region.cloned()),
            signing_service_override.or_else(|| signing_service.cloned()),
        ) {
            (None, None) => Ok(Cow::Borrowed(operation_config)),
            (region, service) => {
                let mut operation_config = operation_config.clone();
                if region.is_some() {
                    operation_config.region = region;
                }
                if service.is_some() {
                    operation_config.service = service;
                }
                Ok(Cow::Owned(operation_config))
            }
        }
    }

    fn extract_endpoint_auth_scheme_config(
        endpoint_config: AuthSchemeEndpointConfig<'_>,
    ) -> Result<EndpointAuthSchemeConfig, SigV4SigningError> {
        let (mut signing_region_override, mut signing_service_override) = (None, None);
        if let Some(config) = endpoint_config.as_document().and_then(Document::as_object) {
            use SigV4SigningError::BadTypeInEndpointAuthSchemeConfig as UnexpectedType;
            signing_region_override = match config.get("signingRegion") {
                Some(Document::String(s)) => Some(SigningRegion::from(Region::new(s.clone()))),
                None => None,
                _ => return Err(UnexpectedType("signingRegion")),
            };
            signing_service_override = match config.get("signingName") {
                Some(Document::String(s)) => Some(SigningService::from(s.to_string())),
                None => None,
                _ => return Err(UnexpectedType("signingName")),
            };
        }
        Ok(EndpointAuthSchemeConfig {
            signing_region_override,
            signing_service_override,
        })
    }
}

impl Signer for SigV4Signer {
    fn sign_http_request(
        &self,
        request: &mut HttpRequest,
        identity: &Identity,
        auth_scheme_endpoint_config: AuthSchemeEndpointConfig<'_>,
        runtime_components: &RuntimeComponents,
        config_bag: &ConfigBag,
    ) -> Result<(), BoxError> {
        let operation_config =
            Self::extract_operation_config(auth_scheme_endpoint_config, config_bag)?;
        let request_time = runtime_components.time_source().unwrap_or_default().now();

        let credentials = if let Some(creds) = identity.data::<Credentials>() {
            creds
        } else if operation_config.signing_options.signing_optional {
            tracing::debug!("skipped SigV4 signing since signing is optional for this operation and there are no credentials");
            return Ok(());
        } else {
            return Err(SigV4SigningError::WrongIdentityType(identity.clone()).into());
        };

        let settings = Self::settings(&operation_config);
        let signing_params =
            Self::signing_params(settings, credentials, &operation_config, request_time)?;

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

        // If this is an event stream operation, set up the event stream signer
        #[cfg(feature = "event-stream")]
        {
            use aws_smithy_eventstream::frame::DeferredSignerSender;
            use event_stream::SigV4MessageSigner;

            if let Some(signer_sender) = config_bag.load::<DeferredSignerSender>() {
                let time_source = runtime_components.time_source().unwrap_or_default();
                signer_sender
                    .send(Box::new(SigV4MessageSigner::new(
                        _signature,
                        credentials.clone(),
                        Region::new(signing_params.region().to_string()).into(),
                        signing_params.service_name().to_string().into(),
                        time_source,
                    )) as _)
                    .expect("failed to send deferred signer");
            }
        }

        signing_instructions.apply_to_request(request);
        Ok(())
    }
}

#[cfg(feature = "event-stream")]
mod event_stream {
    use aws_credential_types::Credentials;
    use aws_sigv4::event_stream::{sign_empty_message, sign_message};
    use aws_sigv4::SigningParams;
    use aws_smithy_async::time::SharedTimeSource;
    use aws_smithy_eventstream::frame::{Message, SignMessage, SignMessageError};
    use aws_types::region::SigningRegion;
    use aws_types::SigningService;

    /// Event Stream SigV4 signing implementation.
    #[derive(Debug)]
    pub(super) struct SigV4MessageSigner {
        last_signature: String,
        credentials: Credentials,
        signing_region: SigningRegion,
        signing_service: SigningService,
        time: SharedTimeSource,
    }

    impl SigV4MessageSigner {
        pub(super) fn new(
            last_signature: String,
            credentials: Credentials,
            signing_region: SigningRegion,
            signing_service: SigningService,
            time: SharedTimeSource,
        ) -> Self {
            Self {
                last_signature,
                credentials,
                signing_region,
                signing_service,
                time,
            }
        }

        fn signing_params(&self) -> SigningParams<'_, ()> {
            let mut builder = SigningParams::builder()
                .access_key(self.credentials.access_key_id())
                .secret_key(self.credentials.secret_access_key())
                .region(self.signing_region.as_ref())
                .service_name(self.signing_service.as_ref())
                .time(self.time.now())
                .settings(());
            builder.set_security_token(self.credentials.session_token());
            builder.build().unwrap()
        }
    }

    impl SignMessage for SigV4MessageSigner {
        fn sign(&mut self, message: Message) -> Result<Message, SignMessageError> {
            let (signed_message, signature) = {
                let params = self.signing_params();
                sign_message(&message, &self.last_signature, &params).into_parts()
            };
            self.last_signature = signature;
            Ok(signed_message)
        }

        fn sign_empty(&mut self) -> Option<Result<Message, SignMessageError>> {
            let (signed_message, signature) = {
                let params = self.signing_params();
                sign_empty_message(&self.last_signature, &params).into_parts()
            };
            self.last_signature = signature;
            Some(Ok(signed_message))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use aws_credential_types::Credentials;
        use aws_smithy_eventstream::frame::{HeaderValue, Message, SignMessage};
        use aws_types::region::Region;
        use aws_types::region::SigningRegion;
        use aws_types::SigningService;
        use std::time::{Duration, UNIX_EPOCH};

        fn check_send_sync<T: Send + Sync>(value: T) -> T {
            value
        }

        #[test]
        fn sign_message() {
            let region = Region::new("us-east-1");
            let mut signer = check_send_sync(SigV4MessageSigner::new(
                "initial-signature".into(),
                Credentials::for_tests(),
                SigningRegion::from(region),
                SigningService::from_static("transcribe"),
                SharedTimeSource::new(UNIX_EPOCH + Duration::new(1611160427, 0)),
            ));
            let mut signatures = Vec::new();
            for _ in 0..5 {
                let signed = signer
                    .sign(Message::new(&b"identical message"[..]))
                    .unwrap();
                if let HeaderValue::ByteArray(signature) = signed
                    .headers()
                    .iter()
                    .find(|h| h.name().as_str() == ":chunk-signature")
                    .unwrap()
                    .value()
                {
                    signatures.push(signature.clone());
                } else {
                    panic!("failed to get the :chunk-signature")
                }
            }
            for i in 1..signatures.len() {
                assert_ne!(signatures[i - 1], signatures[i]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_credential_types::Credentials;
    use aws_sigv4::http_request::SigningSettings;
    use aws_smithy_types::config_bag::Layer;
    use aws_types::region::SigningRegion;
    use aws_types::SigningService;
    use std::collections::HashMap;
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
            region: Some(SigningRegion::from_static("test")),
            service: Some(SigningService::from_static("test")),
            signing_options: SigningOptions {
                double_uri_encode: true,
                content_sha256_header: true,
                normalize_uri_path: true,
                omit_session_token: true,
                signature_type: HttpSignatureType::HttpRequestHeaders,
                signing_optional: false,
                expires_in: None,
                payload_override: None,
            },
        };
        SigV4Signer::signing_params(settings, &credentials, &operation_config, now).unwrap();
        assert!(!logs_contain(EXPIRATION_WARNING));

        let mut settings = SigningSettings::default();
        settings.expires_in = Some(creds_expire_in + Duration::from_secs(10));

        SigV4Signer::signing_params(settings, &credentials, &operation_config, now).unwrap();
        assert!(logs_contain(EXPIRATION_WARNING));
    }

    #[test]
    fn endpoint_config_overrides_region_and_service() {
        let mut layer = Layer::new("test");
        layer.store_put(SigV4OperationSigningConfig {
            region: Some(SigningRegion::from(Region::new("override-this-region"))),
            service: Some(SigningService::from_static("override-this-service")),
            signing_options: Default::default(),
        });
        let config = Document::Object({
            let mut out = HashMap::new();
            out.insert("name".to_string(), "sigv4".to_string().into());
            out.insert(
                "signingName".to_string(),
                "qldb-override".to_string().into(),
            );
            out.insert(
                "signingRegion".to_string(),
                "us-east-override".to_string().into(),
            );
            out
        });
        let config = AuthSchemeEndpointConfig::from(Some(&config));

        let cfg = ConfigBag::of_layers(vec![layer]);
        let result = SigV4Signer::extract_operation_config(config, &cfg).expect("success");

        assert_eq!(
            result.region,
            Some(SigningRegion::from(Region::new("us-east-override")))
        );
        assert_eq!(
            result.service,
            Some(SigningService::from_static("qldb-override"))
        );
        assert!(matches!(result, Cow::Owned(_)));
    }

    #[test]
    fn endpoint_config_supports_fallback_when_region_or_service_are_unset() {
        let mut layer = Layer::new("test");
        layer.store_put(SigV4OperationSigningConfig {
            region: Some(SigningRegion::from(Region::new("us-east-1"))),
            service: Some(SigningService::from_static("qldb")),
            signing_options: Default::default(),
        });
        let cfg = ConfigBag::of_layers(vec![layer]);
        let config = AuthSchemeEndpointConfig::empty();

        let result = SigV4Signer::extract_operation_config(config, &cfg).expect("success");

        assert_eq!(
            result.region,
            Some(SigningRegion::from(Region::new("us-east-1")))
        );
        assert_eq!(result.service, Some(SigningService::from_static("qldb")));
        assert!(matches!(result, Cow::Borrowed(_)));
    }
}

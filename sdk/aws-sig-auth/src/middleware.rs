/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::error::Error;
use std::fmt::{Display, Formatter};

use aws_smithy_http::middleware::MapRequest;
use aws_smithy_http::operation::Request;
use aws_smithy_http::property_bag::PropertyBag;

use aws_credential_types::Credentials;
use aws_sigv4::http_request::SignableBody;
use aws_smithy_async::time::SharedTimeSource;
use aws_types::region::SigningRegion;
use aws_types::SigningService;

use crate::signer::{
    OperationSigningConfig, RequestConfig, SigV4Signer, SigningError, SigningRequirements,
};

#[cfg(feature = "sign-eventstream")]
use crate::event_stream::SigV4MessageSigner as EventStreamSigV4Signer;
#[cfg(feature = "sign-eventstream")]
use aws_smithy_eventstream::frame::DeferredSignerSender;

// TODO(enableNewSmithyRuntimeCleanup): Delete `Signature` when switching to the orchestrator
/// Container for the request signature for use in the property bag.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Signature(String);

impl Signature {
    pub fn new(signature: String) -> Self {
        Self(signature)
    }
}

impl AsRef<str> for Signature {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Middleware stage to sign requests with SigV4
///
/// SigV4RequestSignerStage will load configuration from the request property bag and add
/// a signature.
///
/// Prior to signing, the following fields MUST be present in the property bag:
/// - [`SigningRegion`](SigningRegion): The region used when signing the request, e.g. `us-east-1`
/// - [`SigningService`](SigningService): The name of the service to use when signing the request, e.g. `dynamodb`
/// - [`Credentials`](Credentials): Credentials to sign with
/// - [`OperationSigningConfig`](OperationSigningConfig): Operation specific signing configuration, e.g.
///   changes to URL encoding behavior, or headers that must be omitted.
/// - [`SharedTimeSource`]: The time source to use when signing the request.
/// If any of these fields are missing, the middleware will return an error.
#[derive(Clone, Debug)]
pub struct SigV4SigningStage {
    signer: SigV4Signer,
}

impl SigV4SigningStage {
    pub fn new(signer: SigV4Signer) -> Self {
        Self { signer }
    }
}

#[derive(Debug)]
enum SigningStageErrorKind {
    MissingCredentials,
    MissingSigningRegion,
    MissingSigningService,
    MissingSigningConfig,
    SigningFailure(SigningError),
}

#[derive(Debug)]
pub struct SigningStageError {
    kind: SigningStageErrorKind,
}

impl Display for SigningStageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use SigningStageErrorKind::*;
        match self.kind {
            MissingCredentials => {
                write!(f, "no credentials in the property bag")
            }
            MissingSigningRegion => {
                write!(f, "no signing region in the property bag")
            }
            MissingSigningService => {
                write!(f, "no signing service in the property bag")
            }
            MissingSigningConfig => {
                write!(f, "no signing configuration in the property bag")
            }
            SigningFailure(_) => write!(f, "signing failed"),
        }
    }
}

impl Error for SigningStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use SigningStageErrorKind as ErrorKind;
        match &self.kind {
            ErrorKind::SigningFailure(err) => Some(err),
            ErrorKind::MissingCredentials
            | ErrorKind::MissingSigningRegion
            | ErrorKind::MissingSigningService
            | ErrorKind::MissingSigningConfig => None,
        }
    }
}

impl From<SigningStageErrorKind> for SigningStageError {
    fn from(kind: SigningStageErrorKind) -> Self {
        Self { kind }
    }
}

impl From<SigningError> for SigningStageError {
    fn from(error: SigningError) -> Self {
        Self {
            kind: SigningStageErrorKind::SigningFailure(error),
        }
    }
}

/// Extract a signing config from a [`PropertyBag`](aws_smithy_http::property_bag::PropertyBag)
fn signing_config(
    config: &PropertyBag,
) -> Result<(&OperationSigningConfig, RequestConfig, Credentials), SigningStageError> {
    let operation_config = config
        .get::<OperationSigningConfig>()
        .ok_or(SigningStageErrorKind::MissingSigningConfig)?;
    let credentials = config
        .get::<Credentials>()
        .ok_or(SigningStageErrorKind::MissingCredentials)?
        .clone();
    let region = config
        .get::<SigningRegion>()
        .ok_or(SigningStageErrorKind::MissingSigningRegion)?;
    let signing_service = config
        .get::<SigningService>()
        .ok_or(SigningStageErrorKind::MissingSigningService)?;
    let payload_override = config.get::<SignableBody<'static>>();
    let request_config = RequestConfig {
        request_ts: config
            .get::<SharedTimeSource>()
            .map(|t| t.now())
            .unwrap_or_else(|| SharedTimeSource::default().now()),
        region,
        payload_override,
        service: signing_service,
    };
    Ok((operation_config, request_config, credentials))
}

impl MapRequest for SigV4SigningStage {
    type Error = SigningStageError;

    fn name(&self) -> &'static str {
        "sigv4_sign_request"
    }

    fn apply(&self, req: Request) -> Result<Request, Self::Error> {
        req.augment(|mut req, config| {
            let operation_config = config
                .get::<OperationSigningConfig>()
                .ok_or(SigningStageErrorKind::MissingSigningConfig)?;
            let (operation_config, request_config, creds) =
                match &operation_config.signing_requirements {
                    SigningRequirements::Disabled => return Ok(req),
                    SigningRequirements::Optional => match signing_config(config) {
                        Ok(parts) => parts,
                        Err(_) => return Ok(req),
                    },
                    SigningRequirements::Required => signing_config(config)?,
                };

            let signature = self
                .signer
                .sign(operation_config, &request_config, &creds, &mut req)
                .map_err(SigningStageErrorKind::SigningFailure)?;

            // If this is an event stream operation, set up the event stream signer
            #[cfg(feature = "sign-eventstream")]
            if let Some(signer_sender) = config.get::<DeferredSignerSender>() {
                let time_override = config.get::<SharedTimeSource>().map(|ts| ts.now());
                signer_sender
                    .send(Box::new(EventStreamSigV4Signer::new(
                        signature.as_ref().into(),
                        creds,
                        request_config.region.clone(),
                        request_config.service.clone(),
                        time_override,
                    )) as _)
                    .expect("failed to send deferred signer");
            }

            config.insert(signature);
            Ok(req)
        })
    }
}

#[cfg(test)]
mod test {
    use std::convert::Infallible;
    use std::time::{Duration, UNIX_EPOCH};

    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::middleware::MapRequest;
    use aws_smithy_http::operation;
    use http::header::AUTHORIZATION;

    use aws_credential_types::Credentials;
    use aws_endpoint::AwsAuthStage;
    use aws_smithy_async::time::SharedTimeSource;
    use aws_types::region::{Region, SigningRegion};
    use aws_types::SigningService;

    use crate::middleware::{
        SigV4SigningStage, Signature, SigningStageError, SigningStageErrorKind,
    };
    use crate::signer::{OperationSigningConfig, SigV4Signer};

    #[test]
    fn places_signature_in_property_bag() {
        let req = http::Request::builder()
            .uri("https://test-service.test-region.amazonaws.com/")
            .body(SdkBody::from(""))
            .unwrap();
        let region = Region::new("us-east-1");
        let req = operation::Request::new(req)
            .augment(|req, properties| {
                properties.insert(region.clone());
                properties.insert(UNIX_EPOCH + Duration::new(1611160427, 0));
                properties.insert(SigningService::from_static("kinesis"));
                properties.insert(OperationSigningConfig::default_config());
                properties.insert(Credentials::for_tests());
                properties.insert(SigningRegion::from(region));
                Result::<_, Infallible>::Ok(req)
            })
            .expect("succeeds");

        let signer = SigV4SigningStage::new(SigV4Signer::new());
        let req = signer.apply(req).unwrap();

        let property_bag = req.properties();
        let signature = property_bag.get::<Signature>();
        assert!(signature.is_some());
    }

    #[cfg(feature = "sign-eventstream")]
    #[test]
    fn sends_event_stream_signer_for_event_stream_operations() {
        use crate::event_stream::SigV4MessageSigner as EventStreamSigV4Signer;
        use aws_smithy_eventstream::frame::{DeferredSigner, SignMessage};

        let (mut deferred_signer, deferred_signer_sender) = DeferredSigner::new();
        let req = http::Request::builder()
            .uri("https://test-service.test-region.amazonaws.com/")
            .body(SdkBody::from(""))
            .unwrap();
        let region = Region::new("us-east-1");
        let req = operation::Request::new(req)
            .augment(|req, properties| {
                properties.insert(region.clone());
                properties.insert::<SharedTimeSource>(SharedTimeSource::new(
                    UNIX_EPOCH + Duration::new(1611160427, 0),
                ));
                properties.insert(SigningService::from_static("kinesis"));
                properties.insert(OperationSigningConfig::default_config());
                properties.insert(Credentials::for_tests());
                properties.insert(SigningRegion::from(region.clone()));
                properties.insert(deferred_signer_sender);
                Result::<_, Infallible>::Ok(req)
            })
            .expect("succeeds");

        let signer = SigV4SigningStage::new(SigV4Signer::new());
        let _ = signer.apply(req).unwrap();

        let mut signer_for_comparison = EventStreamSigV4Signer::new(
            // This is the expected SigV4 signature for the HTTP request above
            "abac477b4afabf5651079e7b9a0aa6a1a3e356a7418a81d974cdae9d4c8e5441".into(),
            Credentials::for_tests(),
            SigningRegion::from(region),
            SigningService::from_static("kinesis"),
            Some(UNIX_EPOCH + Duration::new(1611160427, 0)),
        );

        let expected_signed_empty = signer_for_comparison.sign_empty().unwrap().unwrap();
        let actual_signed_empty = deferred_signer.sign_empty().unwrap().unwrap();
        assert_eq!(expected_signed_empty, actual_signed_empty);
    }

    // check that the endpoint middleware followed by signing middleware produce the expected result
    #[test]
    fn endpoint_plus_signer() {
        use aws_smithy_types::endpoint::Endpoint;
        let endpoint = Endpoint::builder()
            .url("https://kinesis.us-east-1.amazonaws.com")
            .build();
        let req = http::Request::builder()
            .uri("https://kinesis.us-east-1.amazonaws.com")
            .body(SdkBody::from(""))
            .unwrap();
        let region = SigningRegion::from_static("us-east-1");
        let req = operation::Request::new(req)
            .augment(|req, conf| {
                conf.insert(region.clone());
                conf.insert(SharedTimeSource::new(
                    UNIX_EPOCH + Duration::new(1611160427, 0),
                ));
                conf.insert(SigningService::from_static("kinesis"));
                conf.insert(endpoint);
                Result::<_, Infallible>::Ok(req)
            })
            .expect("succeeds");

        let endpoint = AwsAuthStage;
        let signer = SigV4SigningStage::new(SigV4Signer::new());
        let mut req = endpoint.apply(req).expect("add endpoint should succeed");
        let mut errs = vec![signer
            .apply(req.try_clone().expect("can clone"))
            .expect_err("no signing config")];
        let mut config = OperationSigningConfig::default_config();
        config.signing_options.content_sha256_header = true;
        req.properties_mut().insert(config);
        errs.push(
            signer
                .apply(req.try_clone().expect("can clone"))
                .expect_err("no cred provider"),
        );
        req.properties_mut().insert(Credentials::for_tests());
        let req = signer.apply(req).expect("signing succeeded");
        // make sure we got the correct error types in any order
        assert!(errs.iter().all(|el| matches!(
            el,
            SigningStageError {
                kind: SigningStageErrorKind::MissingCredentials
                    | SigningStageErrorKind::MissingSigningConfig
            }
        )));

        let (req, _) = req.into_parts();
        assert_eq!(
            req.headers()
                .get("x-amz-date")
                .expect("x-amz-date must be present"),
            "20210120T163347Z"
        );
        let auth_header = req
            .headers()
            .get(AUTHORIZATION)
            .expect("auth header must be present")
            .to_str()
            .unwrap();
        assert_eq!(auth_header, "AWS4-HMAC-SHA256 Credential=ANOTREAL/20210120/us-east-1/kinesis/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-security-token, Signature=228edaefb06378ac8d050252ea18a219da66117dd72759f4d1d60f02ebc3db64");
    }
}

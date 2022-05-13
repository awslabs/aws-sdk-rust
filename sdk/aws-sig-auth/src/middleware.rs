/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::signer::{
    OperationSigningConfig, RequestConfig, SigV4Signer, SigningError, SigningRequirements,
};
use aws_sigv4::http_request::SignableBody;
use aws_smithy_http::middleware::MapRequest;
use aws_smithy_http::operation::Request;
use aws_smithy_http::property_bag::PropertyBag;
use aws_types::region::SigningRegion;
use aws_types::Credentials;
use aws_types::SigningService;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;

/// Container for the request signature for use in the property bag.
#[non_exhaustive]
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
/// If any of these fields are missing, the middleware will return an error.
///
/// The following fields MAY be present in the property bag:
/// - [`SystemTime`](SystemTime): The timestamp to use when signing the request. If this field is not present
///   [`SystemTime::now`](SystemTime::now) will be used.
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
pub enum SigningStageError {
    MissingCredentials,
    MissingSigningRegion,
    MissingSigningService,
    MissingSigningConfig,
    InvalidBodyType,
    SigningFailure(SigningError),
}

impl Display for SigningStageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SigningStageError::MissingCredentials => {
                write!(f, "No credentials in the property bag")
            }
            SigningStageError::MissingSigningRegion => {
                write!(f, "No signing region in the property bag")
            }
            SigningStageError::MissingSigningService => {
                write!(f, "No signing service in the property bag")
            }
            SigningStageError::MissingSigningConfig => {
                write!(f, "No signing configuration in the property bag")
            }
            SigningStageError::InvalidBodyType => write!(
                f,
                "The request body could not be signed by this configuration"
            ),
            SigningStageError::SigningFailure(_) => write!(f, "Signing failed"),
        }
    }
}

impl From<SigningError> for SigningStageError {
    fn from(error: SigningError) -> Self {
        Self::SigningFailure(error)
    }
}

impl Error for SigningStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            SigningStageError::SigningFailure(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

/// Extract a signing config from a [`PropertyBag`](aws_smithy_http::property_bag::PropertyBag)
fn signing_config(
    config: &PropertyBag,
) -> Result<(&OperationSigningConfig, RequestConfig, Credentials), SigningStageError> {
    let operation_config = config
        .get::<OperationSigningConfig>()
        .ok_or(SigningStageError::MissingSigningConfig)?;
    let credentials = config
        .get::<Credentials>()
        .ok_or(SigningStageError::MissingCredentials)?
        .clone();
    let region = config
        .get::<SigningRegion>()
        .ok_or(SigningStageError::MissingSigningRegion)?;
    let signing_service = config
        .get::<SigningService>()
        .ok_or(SigningStageError::MissingSigningService)?;
    let payload_override = config.get::<SignableBody<'static>>();
    let request_config = RequestConfig {
        request_ts: config
            .get::<SystemTime>()
            .copied()
            .unwrap_or_else(SystemTime::now),
        region,
        payload_override,
        service: signing_service,
    };
    Ok((operation_config, request_config, credentials))
}

impl MapRequest for SigV4SigningStage {
    type Error = SigningStageError;

    fn apply(&self, req: Request) -> Result<Request, Self::Error> {
        req.augment(|mut req, config| {
            let operation_config = config
                .get::<OperationSigningConfig>()
                .ok_or(SigningStageError::MissingSigningConfig)?;
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
                .map_err(|err| SigningStageError::SigningFailure(err))?;
            config.insert(signature);
            Ok(req)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::middleware::{SigV4SigningStage, Signature, SigningStageError};
    use crate::signer::{OperationSigningConfig, SigV4Signer};
    use aws_endpoint::partition::endpoint::{Protocol, SignatureVersion};
    use aws_endpoint::{set_endpoint_resolver, AwsEndpointStage};
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::middleware::MapRequest;
    use aws_smithy_http::operation;
    use aws_types::region::{Region, SigningRegion};
    use aws_types::Credentials;
    use aws_types::SigningService;
    use http::header::AUTHORIZATION;
    use std::convert::Infallible;
    use std::sync::Arc;
    use std::time::{Duration, UNIX_EPOCH};

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
                properties.insert(Credentials::new("AKIAfoo", "bar", None, None, "test"));
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

    // check that the endpoint middleware followed by signing middleware produce the expected result
    #[test]
    fn endpoint_plus_signer() {
        let provider = Arc::new(aws_endpoint::partition::endpoint::Metadata {
            uri_template: "kinesis.{region}.amazonaws.com",
            protocol: Protocol::Https,
            credential_scope: Default::default(),
            signature_versions: SignatureVersion::V4,
        });
        let req = http::Request::new(SdkBody::from(""));
        let region = Region::new("us-east-1");
        let req = operation::Request::new(req)
            .augment(|req, conf| {
                conf.insert(region.clone());
                conf.insert(UNIX_EPOCH + Duration::new(1611160427, 0));
                conf.insert(SigningService::from_static("kinesis"));
                set_endpoint_resolver(conf, provider);
                Result::<_, Infallible>::Ok(req)
            })
            .expect("succeeds");

        let endpoint = AwsEndpointStage;
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
        req.properties_mut()
            .insert(Credentials::new("AKIAfoo", "bar", None, None, "test"));
        let req = signer.apply(req).expect("signing succeeded");
        // make sure we got the correct error types in any order
        assert!(errs.iter().all(|el| matches!(
            el,
            SigningStageError::MissingCredentials | SigningStageError::MissingSigningConfig
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
            .expect("auth header must be present");
        assert_eq!(auth_header, "AWS4-HMAC-SHA256 Credential=AKIAfoo/20210120/us-east-1/kinesis/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature=af71a409f0229dfd6e88409cd1b11f5c2803868d6869888e53bbf9ee12a97ea0");
    }
}

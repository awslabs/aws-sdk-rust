/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::signer::{OperationSigningConfig, RequestConfig, SigV4Signer, SigningError};
use aws_auth::{Credentials, CredentialsError, CredentialsProvider};
use smithy_http::middleware::MapRequest;
use smithy_http::operation::Request;
use smithy_http::property_bag::PropertyBag;
use std::time::SystemTime;

/// Middleware stage to sign requests with SigV4
///
/// SigV4RequestSignerStage will load configuration from the request property bag and add
/// a signature.
///
/// Prior to signing, the following fields MUST be present in the property bag:
/// - [`SigningRegion`](SigningRegion): The region used when signing the request, eg. `us-east-1`
/// - [`SigningService`](SigningService): The name of the service to use when signing the request, eg. `dynamodb`
/// - [`CredentialsProvider`](CredentialsProvider): A credentials provider to retrieve credentials
/// - [`OperationSigningConfig`](OperationSigningConfig): Operation specific signing configuration, eg.
///   changes to URL encoding behavior, or headers that must be omitted.
/// If any of these fields are missing, the middleware will return an error.
///
/// The following fields MAY be present in the property bag:
/// - [`SystemTime`](SystemTime): The timestamp to use when signing the request. If this field is not present
///   [`SystemTime::now`](SystemTime::now) will be used.
#[derive(Clone)]
pub struct SigV4SigningStage {
    signer: SigV4Signer,
}

impl SigV4SigningStage {
    pub fn new(signer: SigV4Signer) -> Self {
        Self { signer }
    }
}

use aws_types::region::SigningRegion;
use aws_types::SigningService;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SigningStageError {
    #[error("No credentials provider in the property bag")]
    MissingCredentialsProvider,
    #[error("No signing region in the property bag")]
    MissingSigningRegion,
    #[error("No signing service in the property bag")]
    MissingSigningService,
    #[error("No signing configuration in the property bag")]
    MissingSigningConfig,
    #[error("The request body could not be signed by this configuration")]
    InvalidBodyType,
    #[error("Signing failed")]
    SigningFailure(#[from] SigningError),
    #[error("Failed to load credentials from the credentials provider")]
    CredentialsLoadingError(#[from] CredentialsError),
}

/// Extract a signing config from a [`PropertyBag`](smithy_http::property_bag::PropertyBag)
fn signing_config(
    config: &PropertyBag,
) -> Result<(&OperationSigningConfig, RequestConfig, Credentials), SigningStageError> {
    let operation_config = config
        .get::<OperationSigningConfig>()
        .ok_or(SigningStageError::MissingSigningConfig)?;
    let cred_provider = config
        .get::<CredentialsProvider>()
        .ok_or(SigningStageError::MissingCredentialsProvider)?;
    let creds = cred_provider.provide_credentials()?;
    let region = config
        .get::<SigningRegion>()
        .ok_or(SigningStageError::MissingSigningRegion)?;
    let signing_service = config
        .get::<SigningService>()
        .ok_or(SigningStageError::MissingSigningService)?;
    let request_config = RequestConfig {
        request_ts: config
            .get::<SystemTime>()
            .copied()
            .unwrap_or_else(SystemTime::now),
        region,
        service: signing_service,
    };
    Ok((operation_config, request_config, creds))
}

impl MapRequest for SigV4SigningStage {
    type Error = SigningStageError;

    fn apply(&self, req: Request) -> Result<Request, Self::Error> {
        req.augment(|mut req, config| {
            let (operation_config, request_config, creds) = signing_config(config)?;

            self.signer
                .sign(&operation_config, &request_config, &creds, &mut req)
                .map_err(|err| SigningStageError::SigningFailure(err))?;
            Ok(req)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::middleware::{SigV4SigningStage, SigningStageError};
    use crate::signer::{OperationSigningConfig, SigV4Signer};
    use aws_auth::CredentialsProvider;
    use aws_endpoint::partition::endpoint::{Protocol, SignatureVersion};
    use aws_endpoint::{set_endpoint_resolver, AwsEndpointStage};
    use aws_types::region::Region;
    use aws_types::SigningService;
    use http::header::AUTHORIZATION;
    use smithy_http::body::SdkBody;
    use smithy_http::middleware::MapRequest;
    use smithy_http::operation;
    use std::convert::Infallible;
    use std::sync::Arc;
    use std::time::{Duration, UNIX_EPOCH};

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
        let mut errs = vec![];
        errs.push(
            signer
                .apply(req.try_clone().expect("can clone"))
                .expect_err("no signing config"),
        );
        let mut config = OperationSigningConfig::default_config();
        config.signing_options.content_sha256_header = true;
        req.config_mut().insert(config);
        errs.push(
            signer
                .apply(req.try_clone().expect("can clone"))
                .expect_err("no cred provider"),
        );
        let cred_provider: CredentialsProvider =
            Arc::new(aws_auth::Credentials::from_keys("AKIAfoo", "bar", None));
        req.config_mut().insert(cred_provider);
        let req = signer.apply(req).expect("signing succeeded");
        // make sure we got the correct error types in any order
        assert!(errs.iter().all(|el| matches!(
            el,
            SigningStageError::MissingCredentialsProvider | SigningStageError::MissingSigningConfig
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

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Generalized HTTP credential provider. Currently, this cannot be used directly and can only
//! be used via the ECS credential provider.
//!
//! Future work will stabilize this interface and enable it to be used directly.

use aws_credential_types::provider::{self, error::CredentialsError};
use aws_credential_types::Credentials;
use aws_smithy_client::erase::DynConnector;
use aws_smithy_client::http_connector::ConnectorSettings;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::operation::{Operation, Request};
use aws_smithy_http::response::ParseStrictResponse;
use aws_smithy_http::result::{SdkError, SdkSuccess};
use aws_smithy_http::retry::ClassifyRetry;
use aws_smithy_types::retry::{ErrorKind, RetryKind};

use crate::connector::expect_connector;
use crate::json_credentials::{parse_json_credentials, JsonCredentials, RefreshableCredentials};
use crate::provider_config::ProviderConfig;

use bytes::Bytes;
use http::header::{ACCEPT, AUTHORIZATION};
use http::{HeaderValue, Response, Uri};
use std::time::Duration;
use tower::layer::util::Identity;

const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug)]
pub(crate) struct HttpCredentialProvider {
    uri: Uri,
    client: aws_smithy_client::Client<DynConnector, Identity>,
    provider_name: &'static str,
}

impl HttpCredentialProvider {
    pub(crate) fn builder() -> Builder {
        Builder::default()
    }

    pub(crate) async fn credentials(&self, auth: Option<HeaderValue>) -> provider::Result {
        let credentials = self.client.call(self.operation(auth)).await;
        match credentials {
            Ok(creds) => Ok(creds),
            Err(SdkError::ServiceError(context)) => Err(context.into_err()),
            Err(other) => Err(CredentialsError::unhandled(other)),
        }
    }

    fn operation(
        &self,
        auth: Option<HeaderValue>,
    ) -> Operation<CredentialsResponseParser, HttpCredentialRetryClassifier> {
        let mut http_req = http::Request::builder()
            .uri(&self.uri)
            .header(ACCEPT, "application/json");

        if let Some(auth) = auth {
            http_req = http_req.header(AUTHORIZATION, auth);
        }
        let http_req = http_req.body(SdkBody::empty()).expect("valid request");
        Operation::new(
            Request::new(http_req),
            CredentialsResponseParser {
                provider_name: self.provider_name,
            },
        )
        .with_retry_classifier(HttpCredentialRetryClassifier)
    }
}

#[derive(Default)]
pub(crate) struct Builder {
    provider_config: Option<ProviderConfig>,
    connector_settings: Option<ConnectorSettings>,
}

impl Builder {
    pub(crate) fn configure(mut self, provider_config: &ProviderConfig) -> Self {
        self.provider_config = Some(provider_config.clone());
        self
    }

    pub(crate) fn connector_settings(mut self, connector_settings: ConnectorSettings) -> Self {
        self.connector_settings = Some(connector_settings);
        self
    }

    pub(crate) fn build(self, provider_name: &'static str, uri: Uri) -> HttpCredentialProvider {
        let provider_config = self.provider_config.unwrap_or_default();
        let connector_settings = self.connector_settings.unwrap_or_else(|| {
            ConnectorSettings::builder()
                .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
                .read_timeout(DEFAULT_READ_TIMEOUT)
                .build()
        });
        let connector = expect_connector(
            "The HTTP credentials provider",
            provider_config.connector(&connector_settings),
        );
        let mut client_builder = aws_smithy_client::Client::builder()
            .connector(connector)
            .middleware(Identity::new());
        client_builder.set_sleep_impl(provider_config.sleep());
        let client = client_builder.build();
        HttpCredentialProvider {
            uri,
            client,
            provider_name,
        }
    }
}

#[derive(Clone, Debug)]
struct CredentialsResponseParser {
    provider_name: &'static str,
}
impl ParseStrictResponse for CredentialsResponseParser {
    type Output = provider::Result;

    fn parse(&self, response: &Response<Bytes>) -> Self::Output {
        if !response.status().is_success() {
            return Err(CredentialsError::provider_error(format!(
                "Non-success status from HTTP credential provider: {:?}",
                response.status()
            )));
        }
        let str_resp =
            std::str::from_utf8(response.body().as_ref()).map_err(CredentialsError::unhandled)?;
        let json_creds = parse_json_credentials(str_resp).map_err(CredentialsError::unhandled)?;
        match json_creds {
            JsonCredentials::RefreshableCredentials(RefreshableCredentials {
                access_key_id,
                secret_access_key,
                session_token,
                expiration,
            }) => Ok(Credentials::new(
                access_key_id,
                secret_access_key,
                Some(session_token.to_string()),
                Some(expiration),
                self.provider_name,
            )),
            JsonCredentials::Error { code, message } => Err(CredentialsError::provider_error(
                format!("failed to load credentials [{}]: {}", code, message),
            )),
        }
    }

    fn sensitive(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
struct HttpCredentialRetryClassifier;

impl ClassifyRetry<SdkSuccess<Credentials>, SdkError<CredentialsError>>
    for HttpCredentialRetryClassifier
{
    fn classify_retry(
        &self,
        response: Result<&SdkSuccess<Credentials>, &SdkError<CredentialsError>>,
    ) -> RetryKind {
        /* The following errors are retryable:
         *   - Socket errors
         *   - Networking timeouts
         *   - 5xx errors
         *   - Non-parseable 200 responses.
         *  */
        match response {
            Ok(_) => RetryKind::Unnecessary,
            // socket errors, networking timeouts
            Err(SdkError::DispatchFailure(client_err))
                if client_err.is_timeout() || client_err.is_io() =>
            {
                RetryKind::Error(ErrorKind::TransientError)
            }
            // non-parseable 200s
            Err(SdkError::ServiceError(context))
                if matches!(context.err(), CredentialsError::Unhandled { .. })
                    && context.raw().http().status().is_success() =>
            {
                RetryKind::Error(ErrorKind::ServerError)
            }
            // 5xx errors
            Err(SdkError::ResponseError(context))
                if context.raw().http().status().is_server_error() =>
            {
                RetryKind::Error(ErrorKind::ServerError)
            }
            Err(SdkError::ServiceError(context))
                if context.raw().http().status().is_server_error() =>
            {
                RetryKind::Error(ErrorKind::ServerError)
            }
            Err(_) => RetryKind::UnretryableFailure,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::http_credential_provider::{
        CredentialsResponseParser, HttpCredentialRetryClassifier,
    };
    use aws_credential_types::provider::error::CredentialsError;
    use aws_credential_types::Credentials;
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::operation;
    use aws_smithy_http::response::ParseStrictResponse;
    use aws_smithy_http::result::{SdkError, SdkSuccess};
    use aws_smithy_http::retry::ClassifyRetry;
    use aws_smithy_types::retry::{ErrorKind, RetryKind};
    use bytes::Bytes;

    fn sdk_resp(
        resp: http::Response<&'static str>,
    ) -> Result<SdkSuccess<Credentials>, SdkError<CredentialsError>> {
        let resp = resp.map(|data| Bytes::from_static(data.as_bytes()));
        match (CredentialsResponseParser {
            provider_name: "test",
        })
        .parse(&resp)
        {
            Ok(creds) => Ok(SdkSuccess {
                raw: operation::Response::new(resp.map(SdkBody::from)),
                parsed: creds,
            }),
            Err(err) => Err(SdkError::service_error(
                err,
                operation::Response::new(resp.map(SdkBody::from)),
            )),
        }
    }

    #[test]
    fn non_parseable_is_retriable() {
        let bad_response = http::Response::builder()
            .status(200)
            .body("notjson")
            .unwrap();

        assert_eq!(
            HttpCredentialRetryClassifier.classify_retry(sdk_resp(bad_response).as_ref()),
            RetryKind::Error(ErrorKind::ServerError)
        );
    }

    #[test]
    fn ok_response_not_retriable() {
        let ok_response = http::Response::builder()
            .status(200)
            .body(
                r#" {
   "AccessKeyId" : "MUA...",
   "SecretAccessKey" : "/7PC5om....",
   "Token" : "AQoDY....=",
   "Expiration" : "2016-02-25T06:03:31Z"
 }"#,
            )
            .unwrap();
        let sdk_result = sdk_resp(ok_response);

        assert_eq!(
            HttpCredentialRetryClassifier.classify_retry(sdk_result.as_ref()),
            RetryKind::Unnecessary
        );

        assert!(sdk_result.is_ok(), "should be ok: {:?}", sdk_result)
    }

    #[test]
    fn explicit_error_not_retriable() {
        let error_response = http::Response::builder()
            .status(400)
            .body(r#"{ "Code": "Error", "Message": "There was a problem, it was your fault" }"#)
            .unwrap();
        let sdk_result = sdk_resp(error_response);
        assert_eq!(
            HttpCredentialRetryClassifier.classify_retry(sdk_result.as_ref()),
            RetryKind::UnretryableFailure
        );
        let sdk_error = sdk_result.expect_err("should be error");

        assert!(
            matches!(
                sdk_error,
                SdkError::ServiceError(ref context) if matches!(context.err(), CredentialsError::ProviderError { .. })
            ),
            "should be provider error: {}",
            sdk_error
        );
    }
}

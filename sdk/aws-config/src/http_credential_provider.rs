/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Generalized HTTP credential provider. Currently, this cannot be used directly and can only
//! be used via the ECS credential provider.
//!
//! Future work will stabilize this interface and enable it to be used directly.

use aws_smithy_client::erase::DynConnector;
use aws_smithy_client::http_connector::HttpSettings;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::operation::{Operation, Request};
use aws_smithy_http::response::ParseStrictResponse;
use aws_smithy_http::result::{SdkError, SdkSuccess};
use aws_smithy_http::retry::ClassifyResponse;
use aws_smithy_types::retry::{ErrorKind, RetryKind};
use aws_smithy_types::timeout::TimeoutConfig;
use aws_types::credentials::CredentialsError;
use aws_types::{credentials, Credentials};

use crate::connector::expect_connector;
use crate::json_credentials::{parse_json_credentials, JsonCredentials};
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
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub async fn credentials(&self, auth: Option<HeaderValue>) -> credentials::Result {
        let credentials = self.client.call(self.operation(auth)).await;
        match credentials {
            Ok(creds) => Ok(creds),
            Err(SdkError::ServiceError { err, .. }) => Err(err),
            Err(other) => Err(CredentialsError::unhandled(other)),
        }
    }

    fn operation(
        &self,
        auth: Option<HeaderValue>,
    ) -> Operation<CredentialsResponseParser, HttpCredentialRetryPolicy> {
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
        .with_retry_policy(HttpCredentialRetryPolicy)
    }
}

#[derive(Default)]
pub(crate) struct Builder {
    provider_config: Option<ProviderConfig>,
    timeout_config: TimeoutConfig,
}

impl Builder {
    pub(crate) fn configure(mut self, provider_config: &ProviderConfig) -> Self {
        self.provider_config = Some(provider_config.clone());
        self
    }

    // read_timeout and connect_timeout accept options to enable easy pass through from
    // other builders
    pub(crate) fn read_timeout(mut self, read_timeout: Option<Duration>) -> Self {
        self.timeout_config = self.timeout_config.with_read_timeout(read_timeout);
        self
    }

    pub(crate) fn connect_timeout(mut self, connect_timeout: Option<Duration>) -> Self {
        self.timeout_config = self.timeout_config.with_connect_timeout(connect_timeout);
        self
    }

    pub(crate) fn build(self, provider_name: &'static str, uri: Uri) -> HttpCredentialProvider {
        let provider_config = self.provider_config.unwrap_or_default();
        let default_timeout_config = TimeoutConfig::new()
            .with_connect_timeout(Some(DEFAULT_CONNECT_TIMEOUT))
            .with_read_timeout(Some(DEFAULT_READ_TIMEOUT));
        let timeout_config = self.timeout_config.take_unset_from(default_timeout_config);
        let http_settings = HttpSettings::default().with_timeout_config(timeout_config);
        let connector = expect_connector(provider_config.connector(&http_settings));
        let client = aws_smithy_client::Builder::new()
            .connector(connector)
            .sleep_impl(provider_config.sleep())
            .build();
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
    type Output = credentials::Result;

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
            JsonCredentials::RefreshableCredentials {
                access_key_id,
                secret_access_key,
                session_token,
                expiration,
            } => Ok(Credentials::new(
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
}

#[derive(Clone, Debug)]
struct HttpCredentialRetryPolicy;

impl ClassifyResponse<SdkSuccess<Credentials>, SdkError<CredentialsError>>
    for HttpCredentialRetryPolicy
{
    fn classify(
        &self,
        response: Result<&SdkSuccess<credentials::Credentials>, &SdkError<CredentialsError>>,
    ) -> RetryKind {
        /* The following errors are retryable:
         *   - Socket errors
         *   - Networking timeouts
         *   - 5xx errors
         *   - Non-parseable 200 responses.
         *  */
        match response {
            Ok(_) => RetryKind::NotRetryable,
            // socket errors, networking timeouts
            Err(SdkError::DispatchFailure(client_err))
                if client_err.is_timeout() || client_err.is_io() =>
            {
                RetryKind::Error(ErrorKind::TransientError)
            }
            // non-parseable 200s
            Err(SdkError::ServiceError {
                err: CredentialsError::Unhandled { .. },
                raw,
            }) if raw.http().status().is_success() => RetryKind::Error(ErrorKind::ServerError),
            // 5xx errors
            Err(SdkError::ServiceError { raw, .. } | SdkError::ResponseError { raw, .. })
                if raw.http().status().is_server_error() =>
            {
                RetryKind::Error(ErrorKind::ServerError)
            }
            Err(_) => RetryKind::NotRetryable,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::http_credential_provider::{CredentialsResponseParser, HttpCredentialRetryPolicy};
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::operation;
    use aws_smithy_http::response::ParseStrictResponse;
    use aws_smithy_http::result::{SdkError, SdkSuccess};
    use aws_smithy_http::retry::ClassifyResponse;
    use aws_smithy_types::retry::{ErrorKind, RetryKind};
    use aws_types::credentials::CredentialsError;
    use aws_types::Credentials;
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
            Err(err) => Err(SdkError::ServiceError {
                err,
                raw: operation::Response::new(resp.map(SdkBody::from)),
            }),
        }
    }

    #[test]
    fn non_parseable_is_retriable() {
        let bad_response = http::Response::builder()
            .status(200)
            .body("notjson")
            .unwrap();

        assert_eq!(
            HttpCredentialRetryPolicy.classify(sdk_resp(bad_response).as_ref()),
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
            HttpCredentialRetryPolicy.classify(sdk_result.as_ref()),
            RetryKind::NotRetryable
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
            HttpCredentialRetryPolicy.classify(sdk_result.as_ref()),
            RetryKind::NotRetryable
        );
        let sdk_error = sdk_result.expect_err("should be error");

        assert!(
            matches!(
                sdk_error,
                SdkError::ServiceError {
                    err: CredentialsError::ProviderError { .. },
                    ..
                }
            ),
            "should be provider error: {}",
            sdk_error
        );
    }
}

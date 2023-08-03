/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Assume credentials for a role through the AWS Security Token Service (STS).

use crate::connector::expect_connector;
use crate::provider_config::ProviderConfig;
use aws_credential_types::cache::CredentialsCache;
use aws_credential_types::provider::{self, error::CredentialsError, future, ProvideCredentials};
use aws_sdk_sts::operation::assume_role::builders::AssumeRoleFluentBuilder;
use aws_sdk_sts::operation::assume_role::AssumeRoleError;
use aws_sdk_sts::types::PolicyDescriptorType;
use aws_sdk_sts::Client as StsClient;
use aws_smithy_client::erase::DynConnector;
use aws_smithy_http::result::SdkError;
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_types::region::Region;
use std::time::Duration;
use tracing::Instrument;

/// Credentials provider that uses credentials provided by another provider to assume a role
/// through the AWS Security Token Service (STS).
///
/// When asked to provide credentials, this provider will first invoke the inner credentials
/// provider to get AWS credentials for STS. Then, it will call STS to get assumed credentials for
/// the desired role.
///
/// # Examples
/// ```no_run
/// use aws_credential_types::Credentials;
/// use aws_config::sts::{AssumeRoleProvider};
/// use aws_types::region::Region;
/// use aws_config::environment;
/// use aws_config::environment::credentials::EnvironmentVariableCredentialsProvider;
/// use std::sync::Arc;
///
/// let provider = AssumeRoleProvider::builder("arn:aws:iam::123456789012:role/demo")
///   .region(Region::from_static("us-east-2"))
///   .session_name("testAR")
///   .build(Arc::new(EnvironmentVariableCredentialsProvider::new()) as Arc<_>);
/// ```
#[derive(Debug)]
pub struct AssumeRoleProvider {
    inner: Inner,
}

#[derive(Debug)]
struct Inner {
    fluent_builder: AssumeRoleFluentBuilder,
}

impl AssumeRoleProvider {
    /// Build a new role-assuming provider for the given role.
    ///
    /// The `role` argument should take the form an Amazon Resource Name (ARN) like
    ///
    /// ```text
    /// arn:aws:iam::123456789012:role/example
    /// ```
    pub fn builder(role: impl Into<String>) -> AssumeRoleProviderBuilder {
        AssumeRoleProviderBuilder::new(role.into())
    }
}

/// A builder for [`AssumeRoleProvider`].
///
/// Construct one through [`AssumeRoleProvider::builder`].
#[derive(Debug)]
pub struct AssumeRoleProviderBuilder {
    role_arn: String,
    external_id: Option<String>,
    session_name: Option<String>,
    region: Option<Region>,
    conf: Option<ProviderConfig>,
    session_length: Option<Duration>,
    policy: Option<String>,
    policy_arns: Option<Vec<PolicyDescriptorType>>,
    credentials_cache: Option<CredentialsCache>,
}

impl AssumeRoleProviderBuilder {
    /// Start a new assume role builder for the given role.
    ///
    /// The `role` argument should take the form an Amazon Resource Name (ARN) like
    ///
    /// ```text
    /// arn:aws:iam::123456789012:role/example
    /// ```
    pub fn new(role: impl Into<String>) -> Self {
        Self {
            role_arn: role.into(),
            external_id: None,
            session_name: None,
            session_length: None,
            region: None,
            conf: None,
            policy: None,
            policy_arns: None,
            credentials_cache: None,
        }
    }

    /// Set a unique identifier that might be required when you assume a role in another account.
    ///
    /// If the administrator of the account to which the role belongs provided you with an external
    /// ID, then provide that value in this parameter. The value can be any string, such as a
    /// passphrase or account number.
    pub fn external_id(mut self, id: impl Into<String>) -> Self {
        self.external_id = Some(id.into());
        self
    }

    /// Set an identifier for the assumed role session.
    ///
    /// Use the role session name to uniquely identify a session when the same role is assumed by
    /// different principals or for different reasons. In cross-account scenarios, the role session
    /// name is visible to, and can be logged by the account that owns the role. The role session
    /// name is also used in the ARN of the assumed role principal.
    pub fn session_name(mut self, name: impl Into<String>) -> Self {
        self.session_name = Some(name.into());
        self
    }

    /// Set an IAM policy in JSON format that you want to use as an inline session policy.
    ///
    /// This parameter is optional
    /// For more information, see
    /// [policy](aws_sdk_sts::operation::assume_role::builders::AssumeRoleInputBuilder::policy_arns)
    pub fn policy(mut self, policy: impl Into<String>) -> Self {
        self.policy = Some(policy.into());
        self
    }

    /// Set the Amazon Resource Names (ARNs) of the IAM managed policies that you want to use as managed session policies.
    ///
    /// This parameter is optional.
    /// For more information, see
    /// [policy_arns](aws_sdk_sts::operation::assume_role::builders::AssumeRoleInputBuilder::policy_arns)
    pub fn policy_arns(mut self, policy_arns: Vec<PolicyDescriptorType>) -> Self {
        self.policy_arns = Some(policy_arns);
        self
    }

    /// Set the expiration time of the role session.
    ///
    /// When unset, this value defaults to 1 hour.
    ///
    /// The value specified can range from 900 seconds (15 minutes) up to the maximum session duration
    /// set for the role. The maximum session duration setting can have a value from 1 hour to 12 hours.
    /// If you specify a value higher than this setting or the administrator setting (whichever is lower),
    /// **you will be unable to assume the role**. For example, if you specify a session duration of 12 hours,
    /// but your administrator set the maximum session duration to 6 hours, you cannot assume the role.
    ///
    /// For more information, see
    /// [duration_seconds](aws_sdk_sts::operation::assume_role::builders::AssumeRoleInputBuilder::duration_seconds)
    pub fn session_length(mut self, length: Duration) -> Self {
        self.session_length = Some(length);
        self
    }

    /// Set the region to assume the role in.
    ///
    /// This dictates which STS endpoint the AssumeRole action is invoked on.
    pub fn region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    /// If the `rustls` or `nativetls` features are enabled, this field is optional and a default
    /// backing connection will be provided.
    pub fn connection(mut self, conn: impl aws_smithy_client::bounds::SmithyConnector) -> Self {
        let conf = match self.conf {
            Some(conf) => conf.with_http_connector(DynConnector::new(conn)),
            None => ProviderConfig::default().with_http_connector(DynConnector::new(conn)),
        };
        self.conf = Some(conf);
        self
    }

    #[deprecated(
        note = "This should not be necessary as the default, no caching, is usually what you want."
    )]
    /// Set the [`CredentialsCache`] for credentials retrieved from STS.
    ///
    /// By default, an [`AssumeRoleProvider`] internally uses `NoCredentialsCache` because the
    /// provider itself will be wrapped by `LazyCredentialsCache` when a service client is created.
    pub fn credentials_cache(mut self, cache: CredentialsCache) -> Self {
        self.credentials_cache = Some(cache);
        self
    }

    /// Override the configuration used for this provider
    ///
    /// This enables overriding the connection used to communicate with STS in addition to other internal
    /// fields like the time source and sleep implementation used for caching.
    pub fn configure(mut self, conf: &ProviderConfig) -> Self {
        self.conf = Some(conf.clone());
        self
    }

    /// Build a credentials provider for this role authorized by the given `provider`.
    pub fn build(self, provider: impl ProvideCredentials + 'static) -> AssumeRoleProvider {
        let conf = self.conf.unwrap_or_default();

        let credentials_cache = self
            .credentials_cache
            .unwrap_or_else(CredentialsCache::no_caching);

        let mut config = aws_sdk_sts::Config::builder()
            .credentials_cache(credentials_cache)
            .credentials_provider(provider)
            .time_source(conf.time_source())
            .region(self.region.clone())
            .http_connector(expect_connector(
                "The AssumeRole credentials provider",
                conf.connector(&Default::default()),
            ));
        config.set_sleep_impl(conf.sleep());

        let session_name = self.session_name.unwrap_or_else(|| {
            super::util::default_session_name("assume-role-provider", conf.time_source().now())
        });

        let sts_client = StsClient::from_conf(config.build());
        let fluent_builder = sts_client
            .assume_role()
            .set_role_arn(Some(self.role_arn))
            .set_external_id(self.external_id)
            .set_role_session_name(Some(session_name))
            .set_policy(self.policy)
            .set_policy_arns(self.policy_arns)
            .set_duration_seconds(self.session_length.map(|dur| dur.as_secs() as i32));

        AssumeRoleProvider {
            inner: Inner { fluent_builder },
        }
    }
}

impl Inner {
    async fn credentials(&self) -> provider::Result {
        tracing::debug!("retrieving assumed credentials");

        let assumed = self.fluent_builder.clone().send().in_current_span().await;
        match assumed {
            Ok(assumed) => {
                tracing::debug!(
                    access_key_id = ?assumed.credentials.as_ref().map(|c| &c.access_key_id),
                    "obtained assumed credentials"
                );
                super::util::into_credentials(assumed.credentials, "AssumeRoleProvider")
            }
            Err(SdkError::ServiceError(ref context))
                if matches!(
                    context.err(),
                    AssumeRoleError::RegionDisabledException(_)
                        | AssumeRoleError::MalformedPolicyDocumentException(_)
                ) =>
            {
                Err(CredentialsError::invalid_configuration(
                    assumed.err().unwrap(),
                ))
            }
            Err(SdkError::ServiceError(ref context)) => {
                tracing::warn!(error = %DisplayErrorContext(context.err()), "STS refused to grant assume role");
                Err(CredentialsError::provider_error(assumed.err().unwrap()))
            }
            Err(err) => Err(CredentialsError::provider_error(err)),
        }
    }
}

impl ProvideCredentials for AssumeRoleProvider {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'_>
    where
        Self: 'a,
    {
        future::ProvideCredentials::new(
            self.inner
                .credentials()
                .instrument(tracing::debug_span!("assume_role")),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::provider_config::ProviderConfig;
    use crate::sts::AssumeRoleProvider;
    use aws_credential_types::credential_fn::provide_credentials_fn;
    use aws_credential_types::provider::ProvideCredentials;
    use aws_credential_types::Credentials;
    use aws_smithy_async::rt::sleep::TokioSleep;
    use aws_smithy_async::test_util::instant_time_and_sleep;
    use aws_smithy_async::time::StaticTimeSource;
    use aws_smithy_client::erase::DynConnector;
    use aws_smithy_client::test_connection::{capture_request, TestConnection};
    use aws_smithy_http::body::SdkBody;
    use aws_types::region::Region;
    use std::time::{Duration, UNIX_EPOCH};

    #[tokio::test]
    async fn configures_session_length() {
        let (server, request) = capture_request(None);
        let provider_conf = ProviderConfig::empty()
            .with_sleep(TokioSleep::new())
            .with_time_source(StaticTimeSource::new(
                UNIX_EPOCH + Duration::from_secs(1234567890 - 120),
            ))
            .with_http_connector(DynConnector::new(server));
        let provider = AssumeRoleProvider::builder("myrole")
            .configure(&provider_conf)
            .region(Region::new("us-east-1"))
            .session_length(Duration::from_secs(1234567))
            .build(provide_credentials_fn(|| async {
                Ok(Credentials::for_tests())
            }));
        let _ = provider.provide_credentials().await;
        let req = request.expect_request();
        let str_body = std::str::from_utf8(req.body().bytes().unwrap()).unwrap();
        assert!(str_body.contains("1234567"), "{}", str_body);
    }

    #[tokio::test]
    async fn provider_does_not_cache_credentials_by_default() {
        let conn = TestConnection::new(vec![
            (http::Request::new(SdkBody::from("request body")),
            http::Response::builder().status(200).body(SdkBody::from(
                "<AssumeRoleResponse xmlns=\"https://sts.amazonaws.com/doc/2011-06-15/\">\n  <AssumeRoleResult>\n    <AssumedRoleUser>\n      <AssumedRoleId>AROAR42TAWARILN3MNKUT:assume-role-from-profile-1632246085998</AssumedRoleId>\n      <Arn>arn:aws:sts::130633740322:assumed-role/assume-provider-test/assume-role-from-profile-1632246085998</Arn>\n    </AssumedRoleUser>\n    <Credentials>\n      <AccessKeyId>ASIARCORRECT</AccessKeyId>\n      <SecretAccessKey>secretkeycorrect</SecretAccessKey>\n      <SessionToken>tokencorrect</SessionToken>\n      <Expiration>2009-02-13T23:31:30Z</Expiration>\n    </Credentials>\n  </AssumeRoleResult>\n  <ResponseMetadata>\n    <RequestId>d9d47248-fd55-4686-ad7c-0fb7cd1cddd7</RequestId>\n  </ResponseMetadata>\n</AssumeRoleResponse>\n"
            )).unwrap()),
            (http::Request::new(SdkBody::from("request body")),
            http::Response::builder().status(200).body(SdkBody::from(
                "<AssumeRoleResponse xmlns=\"https://sts.amazonaws.com/doc/2011-06-15/\">\n  <AssumeRoleResult>\n    <AssumedRoleUser>\n      <AssumedRoleId>AROAR42TAWARILN3MNKUT:assume-role-from-profile-1632246085998</AssumedRoleId>\n      <Arn>arn:aws:sts::130633740322:assumed-role/assume-provider-test/assume-role-from-profile-1632246085998</Arn>\n    </AssumedRoleUser>\n    <Credentials>\n      <AccessKeyId>ASIARCORRECT</AccessKeyId>\n      <SecretAccessKey>TESTSECRET</SecretAccessKey>\n      <SessionToken>tokencorrect</SessionToken>\n      <Expiration>2009-02-13T23:33:30Z</Expiration>\n    </Credentials>\n  </AssumeRoleResult>\n  <ResponseMetadata>\n    <RequestId>c2e971c2-702d-4124-9b1f-1670febbea18</RequestId>\n  </ResponseMetadata>\n</AssumeRoleResponse>\n"
            )).unwrap()),
        ]);

        let (testing_time_source, sleep) = instant_time_and_sleep(
            UNIX_EPOCH + Duration::from_secs(1234567890 - 120), // 1234567890 since UNIX_EPOCH is 2009-02-13T23:31:30Z
        );

        let provider_conf = ProviderConfig::empty()
            .with_sleep(sleep)
            .with_time_source(testing_time_source.clone())
            .with_http_connector(DynConnector::new(conn));
        let credentials_list = std::sync::Arc::new(std::sync::Mutex::new(vec![
            Credentials::new(
                "test",
                "test",
                None,
                Some(UNIX_EPOCH + Duration::from_secs(1234567890 + 1)),
                "test",
            ),
            Credentials::new(
                "test",
                "test",
                None,
                Some(UNIX_EPOCH + Duration::from_secs(1234567890 + 120)),
                "test",
            ),
        ]));
        let credentials_list_cloned = credentials_list.clone();
        let provider = AssumeRoleProvider::builder("myrole")
            .configure(&provider_conf)
            .region(Region::new("us-east-1"))
            .build(provide_credentials_fn(move || {
                let list = credentials_list.clone();
                async move {
                    let next = list.lock().unwrap().remove(0);
                    Ok(next)
                }
            }));

        let creds_first = provider
            .provide_credentials()
            .await
            .expect("should return valid credentials");

        // After time has been advanced by 120 seconds, the first credentials _could_ still be valid
        // if `LazyCredentialsCache` were used, but the provider uses `NoCredentialsCache` by default
        // so the first credentials will not be used.
        testing_time_source.advance(Duration::from_secs(120));

        let creds_second = provider
            .provide_credentials()
            .await
            .expect("should return the second credentials");
        assert_ne!(creds_first, creds_second);
        assert!(credentials_list_cloned.lock().unwrap().is_empty());
    }
}

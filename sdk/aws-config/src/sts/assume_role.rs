/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Assume credentials for a role through the AWS Security Token Service (STS).

use aws_sdk_sts::error::AssumeRoleErrorKind;
use aws_sdk_sts::middleware::DefaultMiddleware;
use aws_sdk_sts::operation::AssumeRole;
use aws_smithy_async::rt::sleep::default_async_sleep;
use aws_smithy_client::erase::DynConnector;
use aws_smithy_client::http_connector::HttpSettings;
use aws_smithy_http::result::SdkError;
use aws_types::credentials::{
    self, future, CredentialsError, ProvideCredentials, SharedCredentialsProvider,
};
use aws_types::region::Region;

use crate::connector::{default_connector, expect_connector};
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
/// use aws_config::sts::{AssumeRoleProvider};
/// use aws_types::{Credentials, region::Region};
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
    sts: aws_smithy_client::Client<DynConnector, DefaultMiddleware>,
    conf: aws_sdk_sts::Config,
    op: aws_sdk_sts::input::AssumeRoleInput,
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
pub struct AssumeRoleProviderBuilder {
    role_arn: String,
    external_id: Option<String>,
    session_name: Option<String>,
    region: Option<Region>,
    connection: Option<aws_smithy_client::erase::DynConnector>,
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
            region: None,
            connection: None,
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

    /// Set the region to assume the role in.
    ///
    /// This dictates which STS endpoint the AssumeRole action is invoked on.
    pub fn region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    /// Set the backing connection to use when talking to STS.
    ///
    /// If the `rustls` or `nativetls` features are enabled, this field is optional and a default
    /// backing connection will be provided.
    pub fn connection(mut self, conn: impl aws_smithy_client::bounds::SmithyConnector) -> Self {
        self.connection = Some(aws_smithy_client::erase::DynConnector::new(conn));
        self
    }

    /// Build a credentials provider for this role authorized by the given `provider`.
    pub fn build(self, provider: impl Into<SharedCredentialsProvider>) -> AssumeRoleProvider {
        let config = aws_sdk_sts::Config::builder()
            .credentials_provider(provider.into())
            .region(self.region.clone())
            .build();

        let conn = self.connection.unwrap_or_else(|| {
            expect_connector(default_connector(
                &HttpSettings::default(),
                default_async_sleep(),
            ))
        });
        let client = aws_smithy_client::Builder::new()
            .connector(conn)
            .middleware(DefaultMiddleware::new())
            .sleep_impl(default_async_sleep())
            .build();

        let session_name = self
            .session_name
            .unwrap_or_else(|| super::util::default_session_name("assume-role-provider"));

        let operation = AssumeRole::builder()
            .set_role_arn(Some(self.role_arn))
            .set_external_id(self.external_id)
            .set_role_session_name(Some(session_name))
            .build()
            .expect("operation is valid");

        AssumeRoleProvider {
            sts: client,
            conf: config,
            op: operation,
        }
    }
}

impl AssumeRoleProvider {
    #[tracing::instrument(
        name = "assume_role",
        level = "info",
        skip(self),
        fields(op = ?self.op)
    )]
    async fn credentials(&self) -> credentials::Result {
        tracing::info!("assuming role");

        tracing::debug!("retrieving assumed credentials");
        let op = self
            .op
            .clone()
            .make_operation(&self.conf)
            .await
            .expect("valid operation");

        let assumed = self.sts.call(op).in_current_span().await;
        match assumed {
            Ok(assumed) => {
                tracing::debug!(
                    access_key_id = ?assumed.credentials.as_ref().map(|c| &c.access_key_id),
                    "obtained assumed credentials"
                );
                super::util::into_credentials(assumed.credentials, "AssumeRoleProvider")
            }
            Err(SdkError::ServiceError { err, raw }) => {
                match err.kind {
                    AssumeRoleErrorKind::RegionDisabledException(_)
                    | AssumeRoleErrorKind::MalformedPolicyDocumentException(_) => {
                        return Err(CredentialsError::invalid_configuration(
                            SdkError::ServiceError { err, raw },
                        ))
                    }
                    _ => {}
                }
                tracing::warn!(error = ?err.message(), "sts refused to grant assume role");
                Err(CredentialsError::provider_error(SdkError::ServiceError {
                    err,
                    raw,
                }))
            }
            Err(err) => Err(CredentialsError::provider_error(err)),
        }
    }
}

impl ProvideCredentials for AssumeRoleProvider {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials
    where
        Self: 'a,
    {
        future::ProvideCredentials::new(self.credentials())
    }
}

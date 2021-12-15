/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Credential provider augmentation through the AWS Security Token Service (STS).

mod assume_role;

use crate::connector::expect_connector;
use crate::provider_config::{HttpSettings, ProviderConfig};
pub use assume_role::{AssumeRoleProvider, AssumeRoleProviderBuilder};
use aws_sdk_sts::middleware::DefaultMiddleware;
use aws_smithy_client::erase::DynConnector;

impl ProviderConfig {
    pub(crate) fn sdk_client(&self) -> aws_smithy_client::Client<DynConnector, DefaultMiddleware> {
        aws_smithy_client::Builder::<(), DefaultMiddleware>::new()
            .connector(expect_connector(self.connector(&HttpSettings::default())))
            .sleep_impl(self.sleep())
            .build()
    }
}

pub(crate) mod util {
    use aws_sdk_sts::model::Credentials as StsCredentials;
    use aws_types::credentials::{self, CredentialsError};
    use aws_types::Credentials as AwsCredentials;
    use std::convert::TryFrom;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Convert STS credentials to aws_auth::Credentials
    pub(crate) fn into_credentials(
        sts_credentials: Option<StsCredentials>,
        provider_name: &'static str,
    ) -> credentials::Result {
        let sts_credentials = sts_credentials
            .ok_or_else(|| CredentialsError::unhandled("STS credentials must be defined"))?;
        let expiration = SystemTime::try_from(
            sts_credentials
                .expiration
                .ok_or_else(|| CredentialsError::unhandled("missing expiration"))?,
        )
        .map_err(|_| {
            CredentialsError::unhandled(
                "credential expiration time cannot be represented by a SystemTime",
            )
        })?;
        Ok(AwsCredentials::new(
            sts_credentials
                .access_key_id
                .ok_or_else(|| CredentialsError::unhandled("access key id missing from result"))?,
            sts_credentials
                .secret_access_key
                .ok_or_else(|| CredentialsError::unhandled("secret access token missing"))?,
            sts_credentials.session_token,
            Some(expiration),
            provider_name,
        ))
    }

    /// Create a default STS session name
    ///
    /// STS Assume Role providers MUST assign a name to their generated session. When a user does not
    /// provide a name for the session, the provider will choose a name composed of a base + a timestamp,
    /// e.g. `profile-file-provider-123456789`
    pub(crate) fn default_session_name(base: &str) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("post epoch");
        format!("{}-{}", base, now.as_millis())
    }
}

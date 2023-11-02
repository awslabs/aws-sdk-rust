/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Credentials-based identity support.
pub mod credentials {
    use aws_credential_types::cache::SharedCredentialsCache;
    use aws_smithy_runtime_api::client::identity::{Identity, IdentityFuture, IdentityResolver};
    use aws_smithy_types::config_bag::ConfigBag;

    /// Smithy identity resolver for AWS credentials.
    #[derive(Debug)]
    pub struct CredentialsIdentityResolver {
        credentials_cache: SharedCredentialsCache,
    }

    impl CredentialsIdentityResolver {
        /// Creates a new `CredentialsIdentityResolver`.
        pub fn new(credentials_cache: SharedCredentialsCache) -> Self {
            Self { credentials_cache }
        }
    }

    impl IdentityResolver for CredentialsIdentityResolver {
        fn resolve_identity<'a>(&'a self, _config_bag: &'a ConfigBag) -> IdentityFuture<'a> {
            let cache = self.credentials_cache.clone();
            IdentityFuture::new(async move {
                let credentials = cache.as_ref().provide_cached_credentials().await?;
                let expiration = credentials.expiry();
                Ok(Identity::new(credentials, expiration))
            })
        }
    }
}

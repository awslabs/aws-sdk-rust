/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Credentials-based identity support.
pub mod credentials {
    use aws_credential_types::cache::SharedCredentialsCache;
    use aws_smithy_http::property_bag::PropertyBag;
    use aws_smithy_runtime_api::client::identity::Identity;
    use aws_smithy_runtime_api::client::orchestrator::{
        BoxError, BoxFallibleFut, IdentityResolver,
    };

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
        fn resolve_identity(&self, _identity_properties: &PropertyBag) -> BoxFallibleFut<Identity> {
            let cache = self.credentials_cache.clone();
            Box::pin(async move {
                let credentials = cache.as_ref().provide_cached_credentials().await?;
                let expiration = credentials.expiry();
                Result::<_, BoxError>::Ok(Identity::new(credentials, expiration))
            })
        }
    }
}

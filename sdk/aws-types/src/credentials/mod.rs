/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! AWS SDK Credentials
//!
//! ## Implementing your own credentials provider
//!
//! While for many use cases, using a built in credentials provider is sufficient, you may want to
//! implement your own credential provider.
//!
//! ### With static credentials
//! [`Credentials`](crate::Credentials) implement
//! [`ProvideCredentials](crate::credentials::ProvideCredentials) directly, so no custom provider
//! implementation is required:
//! ```rust
//! use aws_types::Credentials;
//! # mod dynamodb {
//! # use aws_types::credentials::ProvideCredentials;
//! # pub struct Config;
//! # impl Config {
//! #    pub fn builder() -> Self {
//! #        Config
//! #    }
//! #    pub fn credentials_provider(self, provider: impl ProvideCredentials + 'static) -> Self {
//! #       self
//! #    }
//! # }
//! # }
//!
//! let my_creds = Credentials::from_keys("akid", "secret_key", None);
//! let conf = dynamodb::Config::builder().credentials_provider(my_creds);
//! ```
//! ### With dynamically loaded credentials
//! If you are loading credentials dynamically, you can provide your own implementation of
//! [`ProvideCredentials`](crate::credentials::ProvideCredentials). Generally, this is best done by
//! defining an inherent `async fn` on your structure, then calling that method directly from
//! the trait implementation.
//! ```rust
//! use aws_types::credentials::{CredentialsError, Credentials, ProvideCredentials, future, self};
//! #[derive(Debug)]
//! struct SubprocessCredentialProvider;
//!
//! async fn invoke_command(command: &str) -> String {
//!     // implementation elided...
//!     # String::from("some credentials")
//! }
//!
//! /// Parse access key and secret from the first two lines of a string
//! fn parse_credentials(creds: &str) -> credentials::Result {
//!     let mut lines = creds.lines();
//!     let akid = lines.next().ok_or(CredentialsError::ProviderError("invalid credentials".into()))?;
//!     let secret = lines.next().ok_or(CredentialsError::ProviderError("invalid credentials".into()))?;
//!     Ok(Credentials::new(akid, secret, None, None, "CustomCommand"))
//! }
//!
//! impl SubprocessCredentialProvider {
//!     async fn load_credentials(&self) -> credentials::Result {
//!         let creds = invoke_command("load-credentials.py").await;
//!         parse_credentials(&creds)
//!     }
//! }
//!
//! impl ProvideCredentials for SubprocessCredentialProvider {
//!     fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a> where Self: 'a {
//!         future::ProvideCredentials::new(self.load_credentials())
//!     }
//! }
//! ```

mod credentials_impl;
mod provider;

pub use credentials_impl::Credentials;
pub use provider::future;
pub use provider::CredentialsError;
pub use provider::ProvideCredentials;
pub use provider::Result;
pub use provider::SharedCredentialsProvider;

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! AWS SDK Credentials
//!
//! ## Implementing your own credentials provider
//!
//! While for many use cases, using a built in credentials provider is sufficient, you may want to
//! implement your own credential provider.
//!
//! ### With static credentials
//!
//! _Note: In general, you should prefer to use the credential providers that come
//! with the AWS SDK to get credentials. It is __NOT__ secure to hardcode credentials
//! into your application. Only use this approach if you really know what you're doing._
//!
#![cfg_attr(
    feature = "hardcoded-credentials",
    doc = r##"
See [`Credentials::from_keys`] for an example on how to use static credentials.
    "##
)]
#![cfg_attr(
    not(feature = "hardcoded-credentials"),
    doc = r##"
Enable the `hardcoded-credentials` feature to be able to use `Credentials::from_keys` to
construct credentials from hardcoded values.
    "##
)]
//!
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
//!     let akid = lines.next().ok_or(CredentialsError::provider_error("invalid credentials"))?;
//!     let secret = lines.next().ok_or(CredentialsError::provider_error("invalid credentials"))?;
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

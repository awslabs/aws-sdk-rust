/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Credential providers that augment an existing credentials providers to add functionality

mod chain;
pub use chain::CredentialsProviderChain;

mod credential_fn;
pub use credential_fn::provide_credentials_fn;

#[cfg(any(feature = "meta", feature = "default-provider"))]
pub mod lazy_caching;
#[cfg(any(feature = "meta", feature = "default-provider"))]
pub use lazy_caching::LazyCachingCredentialsProvider;

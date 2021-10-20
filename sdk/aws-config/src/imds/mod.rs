/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! IMDSv2 Client, credential, and region provider
//!
//! See [`client`] for more information.
//!
//! _Note: An IMDS credentials provider is not currently implemented. This module currently only
//! contains an IMDS client._
//!
pub mod client;

pub mod credentials;
pub mod region;

mod env {
    pub(crate) const EC2_METADATA_DISABLED: &str = "AWS_EC2_METADATA_DISABLED";
}

#[doc(inline)]
pub use client::Client;

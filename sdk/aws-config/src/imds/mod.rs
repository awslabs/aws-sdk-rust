/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! IMDSv2 Client, credential, and region provider
//!
//! See [`client`] for more information.
pub mod client;

pub mod credentials;
pub mod region;

mod env {
    pub(crate) const EC2_METADATA_DISABLED: &str = "AWS_EC2_METADATA_DISABLED";
    pub(crate) const EC2_INSTANCE_PROFILE_NAME: &str = "AWS_EC2_INSTANCE_PROFILE_NAME";
}

mod profile_key {
    pub(crate) const EC2_METADATA_DISABLED: &str = "disable_ec2_metadata";
    pub(crate) const EC2_INSTANCE_PROFILE_NAME: &str = "ec2_instance_profile_name";
}

#[doc(inline)]
pub use client::Client;

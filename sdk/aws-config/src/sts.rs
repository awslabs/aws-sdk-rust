/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Credential provider augmentation through the AWS Security Token Service (STS).

use crate::connector::expect_connector;
use aws_sdk_sts::middleware::DefaultMiddleware;
use aws_smithy_client::erase::DynConnector;
use aws_smithy_client::Client;

pub(crate) mod util;

pub use assume_role::{AssumeRoleProvider, AssumeRoleProviderBuilder};

mod assume_role;

impl crate::provider_config::ProviderConfig {
    pub(crate) fn sts_client(&self) -> Client<DynConnector, DefaultMiddleware> {
        let mut builder = Client::builder()
            .connector(expect_connector(self.connector(&Default::default())))
            .middleware(DefaultMiddleware::default());
        builder.set_sleep_impl(self.sleep());
        builder.build()
    }
}

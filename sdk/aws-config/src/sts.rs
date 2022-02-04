/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Credential provider augmentation through the AWS Security Token Service (STS).

mod assume_role;

pub(crate) mod util;

use crate::connector::expect_connector;
pub use assume_role::{AssumeRoleProvider, AssumeRoleProviderBuilder};

use aws_sdk_sts::middleware::DefaultMiddleware;
use aws_smithy_client::erase::DynConnector;
use aws_smithy_client::http_connector::HttpSettings;
use aws_smithy_client::{Builder, Client};

impl crate::provider_config::ProviderConfig {
    pub(crate) fn sts_client(&self) -> Client<DynConnector, DefaultMiddleware> {
        Builder::<(), DefaultMiddleware>::new()
            .connector(expect_connector(self.connector(&HttpSettings::default())))
            .sleep_impl(self.sleep())
            .build()
    }
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Types for representing the interaction between a service an a client, referred to as an "operation" in smithy.
//! Clients "send" operations to services, which are composed of 1 or more HTTP requests.

use aws_smithy_types::config_bag::{Storable, StoreReplace};
use std::borrow::Cow;

/// Metadata added to the [`ConfigBag`](aws_smithy_types::config_bag::ConfigBag) that identifies the API being called.
#[derive(Clone, Debug)]
pub struct Metadata {
    operation: Cow<'static, str>,
    service: Cow<'static, str>,
}

impl Metadata {
    /// Returns the operation name.
    pub fn name(&self) -> &str {
        &self.operation
    }

    /// Returns the service name.
    pub fn service(&self) -> &str {
        &self.service
    }

    /// Creates [`Metadata`].
    pub fn new(
        operation: impl Into<Cow<'static, str>>,
        service: impl Into<Cow<'static, str>>,
    ) -> Self {
        Metadata {
            operation: operation.into(),
            service: service.into(),
        }
    }
}

impl Storable for Metadata {
    type Storer = StoreReplace<Self>;
}

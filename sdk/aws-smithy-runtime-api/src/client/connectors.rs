/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::orchestrator::{BoxFuture, HttpRequest, HttpResponse};
use aws_smithy_types::config_bag::{Storable, StoreReplace};
use std::fmt;

pub trait Connector: Send + Sync + fmt::Debug {
    fn call(&self, request: HttpRequest) -> BoxFuture<HttpResponse>;
}

#[derive(Debug)]
pub struct DynConnector(Box<dyn Connector>);

impl DynConnector {
    pub fn new(connection: impl Connector + 'static) -> Self {
        Self(Box::new(connection))
    }
}

impl Connector for DynConnector {
    fn call(&self, request: HttpRequest) -> BoxFuture<HttpResponse> {
        (*self.0).call(request)
    }
}

impl Storable for DynConnector {
    type Storer = StoreReplace<Self>;
}

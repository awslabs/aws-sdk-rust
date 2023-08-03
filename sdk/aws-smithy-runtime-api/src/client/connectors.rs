/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::orchestrator::{BoxFuture, HttpRequest, HttpResponse};
use std::fmt;
use std::sync::Arc;

pub trait Connector: Send + Sync + fmt::Debug {
    fn call(&self, request: HttpRequest) -> BoxFuture<HttpResponse>;
}

#[derive(Clone, Debug)]
pub struct SharedConnector(Arc<dyn Connector>);

impl SharedConnector {
    pub fn new(connection: impl Connector + 'static) -> Self {
        Self(Arc::new(connection))
    }
}

impl Connector for SharedConnector {
    fn call(&self, request: HttpRequest) -> BoxFuture<HttpResponse> {
        (*self.0).call(request)
    }
}

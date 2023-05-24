/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::interceptors::context::Input;
use aws_smithy_runtime_api::client::interceptors::Interceptors;
use aws_smithy_runtime_api::client::orchestrator::{
    ConfigBagAccessors, HttpRequest, RequestSerializer,
};
use aws_smithy_runtime_api::client::runtime_plugin::{BoxError, RuntimePlugin};
use aws_smithy_runtime_api::config_bag::ConfigBag;
use std::sync::Mutex;

#[derive(Default, Debug)]
pub struct CannedRequestSerializer {
    inner: Mutex<Option<Result<HttpRequest, BoxError>>>,
}

impl CannedRequestSerializer {
    pub fn success(request: HttpRequest) -> Self {
        Self {
            inner: Mutex::new(Some(Ok(request))),
        }
    }

    pub fn failure(error: BoxError) -> Self {
        Self {
            inner: Mutex::new(Some(Err(error))),
        }
    }

    pub fn take(&self) -> Option<Result<HttpRequest, BoxError>> {
        match self.inner.lock() {
            Ok(mut guard) => guard.take(),
            Err(_) => None,
        }
    }
}

impl RequestSerializer for CannedRequestSerializer {
    fn serialize_input(&self, _input: Input) -> Result<HttpRequest, BoxError> {
        let req = self
            .take()
            .ok_or("CannedRequestSerializer's inner value has already been taken.")?;
        req
    }
}

impl RuntimePlugin for CannedRequestSerializer {
    fn configure(
        &self,
        cfg: &mut ConfigBag,
        _interceptors: &mut Interceptors,
    ) -> Result<(), BoxError> {
        cfg.set_request_serializer(Self {
            inner: Mutex::new(self.take()),
        });

        Ok(())
    }
}

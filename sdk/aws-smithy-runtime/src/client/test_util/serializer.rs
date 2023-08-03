/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::config_bag_accessors::ConfigBagAccessors;
use aws_smithy_runtime_api::client::interceptors::context::Input;
use aws_smithy_runtime_api::client::orchestrator::SharedRequestSerializer;
use aws_smithy_runtime_api::client::orchestrator::{HttpRequest, RequestSerializer};
use aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin;
use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer, Layer};
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
    fn serialize_input(
        &self,
        _input: Input,
        _cfg: &mut ConfigBag,
    ) -> Result<HttpRequest, BoxError> {
        self.take()
            .ok_or("CannedRequestSerializer's inner value has already been taken.")?
    }
}

impl RuntimePlugin for CannedRequestSerializer {
    fn config(&self) -> Option<FrozenLayer> {
        let mut cfg = Layer::new("CannedRequest");
        cfg.set_request_serializer(SharedRequestSerializer::new(Self {
            inner: Mutex::new(self.take()),
        }));
        Some(cfg.freeze())
    }
}

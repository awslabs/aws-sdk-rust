/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::config_bag_accessors::ConfigBagAccessors;
use aws_smithy_runtime_api::client::interceptors::context::{Error, Output};
use aws_smithy_runtime_api::client::orchestrator::{
    DynResponseDeserializer, HttpResponse, OrchestratorError, ResponseDeserializer,
};
use aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin;
use aws_smithy_types::config_bag::{FrozenLayer, Layer};
use std::sync::Mutex;

#[derive(Default, Debug)]
pub struct CannedResponseDeserializer {
    inner: Mutex<Option<Result<Output, OrchestratorError<Error>>>>,
}

impl CannedResponseDeserializer {
    pub fn new(output: Result<Output, OrchestratorError<Error>>) -> Self {
        Self {
            inner: Mutex::new(Some(output)),
        }
    }

    pub fn take(&self) -> Option<Result<Output, OrchestratorError<Error>>> {
        match self.inner.lock() {
            Ok(mut guard) => guard.take(),
            Err(_) => None,
        }
    }
}

impl ResponseDeserializer for CannedResponseDeserializer {
    fn deserialize_nonstreaming(
        &self,
        _response: &HttpResponse,
    ) -> Result<Output, OrchestratorError<Error>> {
        self.take()
            .ok_or("CannedResponseDeserializer's inner value has already been taken.")
            .unwrap()
    }
}

impl RuntimePlugin for CannedResponseDeserializer {
    fn config(&self) -> Option<FrozenLayer> {
        let mut cfg = Layer::new("CannedResponse");
        cfg.set_response_deserializer(DynResponseDeserializer::new(Self {
            inner: Mutex::new(self.take()),
        }));
        Some(cfg.freeze())
    }
}

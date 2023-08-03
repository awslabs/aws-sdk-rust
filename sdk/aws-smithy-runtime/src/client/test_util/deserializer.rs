/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::interceptors::context::{Error, Output};
use aws_smithy_runtime_api::client::interceptors::InterceptorRegistrar;
use aws_smithy_runtime_api::client::orchestrator::{
    ConfigBagAccessors, HttpResponse, OrchestratorError, ResponseDeserializer,
};
use aws_smithy_runtime_api::client::runtime_plugin::{BoxError, RuntimePlugin};
use aws_smithy_types::config_bag::ConfigBag;
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
    fn configure(
        &self,
        cfg: &mut ConfigBag,
        _interceptors: &mut InterceptorRegistrar,
    ) -> Result<(), BoxError> {
        cfg.set_response_deserializer(Self {
            inner: Mutex::new(self.take()),
        });

        Ok(())
    }
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod connection_poisoning;
#[cfg(feature = "test-util")]
pub mod test_util;

// TODO(enableNewSmithyRuntimeCleanup): Delete this module
/// Unstable API for interfacing the old middleware connectors with the newer orchestrator connectors.
///
/// Important: This module and its contents will be removed in the next release.
pub mod adapter {
    use aws_smithy_client::erase::DynConnector;
    use aws_smithy_runtime_api::client::connectors::HttpConnector;
    use aws_smithy_runtime_api::client::orchestrator::{BoxFuture, HttpRequest, HttpResponse};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    pub struct DynConnectorAdapter {
        // `DynConnector` requires `&mut self`, so we need interior mutability to adapt to it
        dyn_connector: Arc<Mutex<DynConnector>>,
    }

    impl DynConnectorAdapter {
        pub fn new(dyn_connector: DynConnector) -> Self {
            Self {
                dyn_connector: Arc::new(Mutex::new(dyn_connector)),
            }
        }
    }

    impl HttpConnector for DynConnectorAdapter {
        fn call(&self, request: HttpRequest) -> BoxFuture<HttpResponse> {
            let future = self.dyn_connector.lock().unwrap().call_lite(request);
            future
        }
    }
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[cfg(feature = "test-util")]
pub mod test_connection;

pub mod adapter {
    use aws_smithy_client::erase::DynConnector;
    use aws_smithy_runtime_api::client::orchestrator::{
        BoxFallibleFut, Connection, HttpRequest, HttpResponse,
    };
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

    impl Connection for DynConnectorAdapter {
        fn call(&self, request: HttpRequest) -> BoxFallibleFut<HttpResponse> {
            let future = self.dyn_connector.lock().unwrap().call_lite(request);
            future
        }
    }
}

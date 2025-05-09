/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Basic test of using the mock_client macro from an "external" crate

mod fake_crate {
    pub(crate) mod client {
        use crate::fake_crate::config;

        pub(crate) struct Client {}
        impl Client {
            pub(crate) fn from_conf(_conf: config::Config) -> Self {
                Self {}
            }
        }
    }

    pub(crate) mod config {
        use aws_smithy_runtime_api::client::http::SharedHttpClient;
        use aws_smithy_runtime_api::client::interceptors::Intercept;

        pub(crate) struct Config {}
        impl Config {
            pub(crate) fn builder() -> Builder {
                Builder {}
            }
        }
        pub(crate) struct Builder {}
        impl Builder {
            pub fn build(self) -> Config {
                Config {}
            }
            pub fn region(self, _region: crate::fake_crate::config::Region) -> Self {
                Self {}
            }
            pub fn with_test_defaults(self) -> Self {
                Self {}
            }
            pub fn http_client(self, _http_client: SharedHttpClient) -> Self {
                Self {}
            }

            pub fn interceptor(self, _interceptor: impl Intercept + 'static) -> Self {
                self
            }
        }

        pub(crate) struct Region {}
        impl Region {
            pub fn from_static(_region: &'static str) -> Self {
                Self {}
            }
        }
    }
}
#[test]
fn mock_client() {
    aws_smithy_mocks::mock_client!(fake_crate, &[]);
}

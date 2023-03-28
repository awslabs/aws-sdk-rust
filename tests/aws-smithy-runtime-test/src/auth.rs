/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::body::SdkBody;
use aws_smithy_runtime::{AuthOrchestrator, BoxError};
use aws_smithy_runtime_api::config_bag::ConfigBag;
use aws_smithy_runtime_api::runtime_plugin::RuntimePlugin;

#[derive(Debug)]
pub struct GetObjectAuthOrc {}

impl GetObjectAuthOrc {
    pub fn new() -> Self {
        Self {}
    }
}

impl RuntimePlugin for GetObjectAuthOrc {
    fn configure(&self, _cfg: &mut ConfigBag) -> Result<(), BoxError> {
        todo!()
    }
}

impl AuthOrchestrator<http::Request<SdkBody>> for GetObjectAuthOrc {
    fn auth_request(
        &self,
        _req: &mut http::Request<SdkBody>,
        _cfg: &ConfigBag,
    ) -> Result<(), BoxError> {
        todo!()

        // let signer = SigV4Signer::new();
        // let operation_config = props
        //     .get::<OperationSigningConfig>()
        //     .ok_or("missing signing config".to_string())?;
        //
        // let (operation_config, request_config, creds) = match &operation_config.signing_requirements
        // {
        //     SigningRequirements::Disabled => return Ok(()),
        //     SigningRequirements::Optional => {
        //         match aws_sig_auth::middleware::signing_config(props) {
        //             Ok(parts) => parts,
        //             Err(_) => return Ok(()),
        //         }
        //     }
        //     SigningRequirements::Required => {
        //         aws_sig_auth::middleware::signing_config(props).map_err(Box::new)?
        //     }
        // };
        //
        // let _signature = signer
        //     .sign(&operation_config, &request_config, &creds, req)
        //     .expect("signing goes just fine");
        //
        // Ok(())
    }
}

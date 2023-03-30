/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use super::InterceptorError;

/// A container for the data currently available to an interceptor.
pub struct InterceptorContext<ModReq, TxReq, TxRes, ModRes> {
    modeled_request: ModReq,
    tx_request: Option<TxReq>,
    modeled_response: Option<ModRes>,
    tx_response: Option<TxRes>,
}

// TODO(interceptors) we could use types to ensure that people calling methods on interceptor context can't access
//     field that haven't been set yet.
impl<ModReq, TxReq, TxRes, ModRes> InterceptorContext<ModReq, TxReq, TxRes, ModRes> {
    pub fn new(request: ModReq) -> Self {
        Self {
            modeled_request: request,
            tx_request: None,
            tx_response: None,
            modeled_response: None,
        }
    }

    /// Retrieve the modeled request for the operation being invoked.
    pub fn modeled_request(&self) -> &ModReq {
        &self.modeled_request
    }

    /// Retrieve the modeled request for the operation being invoked.
    pub fn modeled_request_mut(&mut self) -> &mut ModReq {
        &mut self.modeled_request
    }

    /// Retrieve the transmittable request for the operation being invoked.
    /// This will only be available once request marshalling has completed.
    pub fn tx_request(&self) -> Result<&TxReq, InterceptorError> {
        self.tx_request
            .as_ref()
            .ok_or_else(InterceptorError::invalid_tx_request_access)
    }

    /// Retrieve the transmittable request for the operation being invoked.
    /// This will only be available once request marshalling has completed.
    pub fn tx_request_mut(&mut self) -> Result<&mut TxReq, InterceptorError> {
        self.tx_request
            .as_mut()
            .ok_or_else(InterceptorError::invalid_tx_request_access)
    }

    /// Retrieve the response to the transmittable request for the operation
    /// being invoked. This will only be available once transmission has
    /// completed.
    pub fn tx_response(&self) -> Result<&TxRes, InterceptorError> {
        self.tx_response
            .as_ref()
            .ok_or_else(InterceptorError::invalid_tx_response_access)
    }

    /// Retrieve the response to the transmittable request for the operation
    /// being invoked. This will only be available once transmission has
    /// completed.
    pub fn tx_response_mut(&mut self) -> Result<&mut TxRes, InterceptorError> {
        self.tx_response
            .as_mut()
            .ok_or_else(InterceptorError::invalid_tx_response_access)
    }

    /// Retrieve the response to the customer. This will only be available
    /// once the `tx_response` has been unmarshalled or the
    /// attempt/execution has failed.
    pub fn modeled_response(&self) -> Result<&ModRes, InterceptorError> {
        self.modeled_response
            .as_ref()
            .ok_or_else(InterceptorError::invalid_modeled_response_access)
    }

    /// Retrieve the response to the customer. This will only be available
    /// once the `tx_response` has been unmarshalled or the
    /// attempt/execution has failed.
    pub fn modeled_response_mut(&mut self) -> Result<&mut ModRes, InterceptorError> {
        self.modeled_response
            .as_mut()
            .ok_or_else(InterceptorError::invalid_modeled_response_access)
    }

    // There is no set_modeled_request method because that can only be set once, during context construction

    pub fn set_tx_request(&mut self, transmit_request: TxReq) {
        if self.tx_request.is_some() {
            panic!("Called set_tx_request but a transmit_request was already set. This is a bug, pleases report it.");
        }

        self.tx_request = Some(transmit_request);
    }

    pub fn set_tx_response(&mut self, transmit_response: TxRes) {
        if self.tx_response.is_some() {
            panic!("Called set_tx_response but a transmit_response was already set. This is a bug, pleases report it.");
        }

        self.tx_response = Some(transmit_response);
    }

    pub fn set_modeled_response(&mut self, modeled_response: ModRes) {
        if self.modeled_response.is_some() {
            panic!("Called set_modeled_response but a modeled_response was already set. This is a bug, pleases report it.");
        }

        self.modeled_response = Some(modeled_response);
    }

    pub fn into_responses(self) -> Result<(ModRes, TxRes), InterceptorError> {
        let mod_res = self
            .modeled_response
            .ok_or_else(InterceptorError::invalid_modeled_response_access)?;
        let tx_res = self
            .tx_response
            .ok_or_else(InterceptorError::invalid_tx_response_access)?;

        Ok((mod_res, tx_res))
    }
}

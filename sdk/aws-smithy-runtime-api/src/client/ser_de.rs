/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Serialization/deserialization for the orchestrator.

use crate::box_error::BoxError;
use crate::client::interceptors::context::{Error, Input, Output};
use crate::client::orchestrator::{HttpRequest, HttpResponse, OrchestratorError};
use crate::impl_shared_conversions;
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreReplace};
use std::fmt;
use std::sync::Arc;

/// Serialization implementation that converts an [`Input`] into an [`HttpRequest`].
pub trait SerializeRequest: Send + Sync + fmt::Debug {
    /// Serializes the input into an HTTP request.
    ///
    /// The type of the [`Input`] must be known ahead of time by the request serializer
    /// implementation, and must be downcasted to get access to the information necessary
    /// for serialization.
    ///
    /// The request serializer is generally added to the [`ConfigBag`] by the operation's
    /// code generated runtime plugin, which is aware of the correct input/output/error types.
    fn serialize_input(&self, input: Input, cfg: &mut ConfigBag) -> Result<HttpRequest, BoxError>;
}

/// A shared request serializer.
///
/// This is a simple shared ownership wrapper type for the [`SerializeRequest`] trait.
#[derive(Clone, Debug)]
pub struct SharedRequestSerializer(Arc<dyn SerializeRequest>);

impl SharedRequestSerializer {
    /// Creates a new shared request serializer.
    pub fn new(serializer: impl SerializeRequest + 'static) -> Self {
        Self(Arc::new(serializer))
    }
}

impl SerializeRequest for SharedRequestSerializer {
    fn serialize_input(&self, input: Input, cfg: &mut ConfigBag) -> Result<HttpRequest, BoxError> {
        self.0.serialize_input(input, cfg)
    }
}

impl Storable for SharedRequestSerializer {
    type Storer = StoreReplace<Self>;
}

impl_shared_conversions!(convert SharedRequestSerializer from SerializeRequest using SharedRequestSerializer::new);

/// Deserialization implementation that converts an [`HttpResponse`] into an [`Output`] or [`Error`].
///
/// This trait uses a backward-compatible versioning pattern for methods that need
/// access to [`ConfigBag`]. Each method has a `_with_config` variant:
///
/// - **Legacy implementations** override the original method (without config).
///   The default `_with_config` variant delegates to it, ignoring the config.
/// - **New implementations** override the `_with_config` variant.
///   The default original method delegates to it with an empty config bag.
///
/// The orchestrator always calls the `_with_config` variants.
pub trait DeserializeResponse: Send + Sync + fmt::Debug {
    /// For streaming requests, deserializes the response headers.
    ///
    /// The orchestrator will call [`deserialize_streaming_with_config`](Self::deserialize_streaming_with_config)
    /// first, and if it returns `None`, then it will continue onto
    /// [`deserialize_nonstreaming_with_config`](Self::deserialize_nonstreaming_with_config).
    ///
    /// Override this or [`deserialize_streaming_with_config`](Self::deserialize_streaming_with_config),
    /// but not both.
    #[deprecated(
        note = "Implement `deserialize_streaming_with_config` instead. This method will be removed in a future release."
    )]
    fn deserialize_streaming(
        &self,
        response: &mut HttpResponse,
    ) -> Option<Result<Output, OrchestratorError<Error>>> {
        let _ = response;
        None
    }

    /// For streaming requests, deserializes the response headers with access to the config bag.
    ///
    /// This is the method called by the orchestrator. The default implementation
    /// delegates to [`deserialize_streaming`](Self::deserialize_streaming), ignoring the config.
    fn deserialize_streaming_with_config(
        &self,
        response: &mut HttpResponse,
        _cfg: &ConfigBag,
    ) -> Option<Result<Output, OrchestratorError<Error>>> {
        #[allow(deprecated)]
        self.deserialize_streaming(response)
    }

    /// Deserialize the entire response including its body into an output or error.
    ///
    /// Override this or [`deserialize_nonstreaming_with_config`](Self::deserialize_nonstreaming_with_config),
    /// but not both.
    #[deprecated(
        note = "Implement `deserialize_nonstreaming_with_config` instead. This method will be removed in a future release."
    )]
    fn deserialize_nonstreaming(
        &self,
        response: &HttpResponse,
    ) -> Result<Output, OrchestratorError<Error>> {
        self.deserialize_nonstreaming_with_config(response, &ConfigBag::base())
    }

    /// Deserialize the entire response including its body into an output or error,
    /// with access to the config bag.
    ///
    /// This is the method called by the orchestrator. The default implementation
    /// delegates to [`deserialize_nonstreaming`](Self::deserialize_nonstreaming),
    /// ignoring the config bag.
    fn deserialize_nonstreaming_with_config(
        &self,
        response: &HttpResponse,
        _cfg: &ConfigBag,
    ) -> Result<Output, OrchestratorError<Error>> {
        #[allow(deprecated)]
        self.deserialize_nonstreaming(response)
    }
}

/// Shared response deserializer.
///
/// This is a simple shared ownership wrapper type for the [`DeserializeResponse`] trait.
#[derive(Debug)]
pub struct SharedResponseDeserializer(Arc<dyn DeserializeResponse>);

impl SharedResponseDeserializer {
    /// Creates a new [`SharedResponseDeserializer`].
    pub fn new(serializer: impl DeserializeResponse + 'static) -> Self {
        Self(Arc::new(serializer))
    }
}

#[allow(deprecated)]
impl DeserializeResponse for SharedResponseDeserializer {
    fn deserialize_nonstreaming(
        &self,
        response: &HttpResponse,
    ) -> Result<Output, OrchestratorError<Error>> {
        self.0.deserialize_nonstreaming(response)
    }

    fn deserialize_nonstreaming_with_config(
        &self,
        response: &HttpResponse,
        cfg: &ConfigBag,
    ) -> Result<Output, OrchestratorError<Error>> {
        self.0.deserialize_nonstreaming_with_config(response, cfg)
    }

    fn deserialize_streaming(
        &self,
        response: &mut HttpResponse,
    ) -> Option<Result<Output, OrchestratorError<Error>>> {
        self.0.deserialize_streaming(response)
    }

    fn deserialize_streaming_with_config(
        &self,
        response: &mut HttpResponse,
        cfg: &ConfigBag,
    ) -> Option<Result<Output, OrchestratorError<Error>>> {
        self.0.deserialize_streaming_with_config(response, cfg)
    }
}

impl Storable for SharedResponseDeserializer {
    type Storer = StoreReplace<Self>;
}

impl_shared_conversions!(convert SharedResponseDeserializer from DeserializeResponse using SharedResponseDeserializer::new);

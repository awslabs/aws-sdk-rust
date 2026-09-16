/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! RPC v2 CBOR protocol implementation.

use crate::codec::{CborCodec, CborCodecSettings};
use aws_smithy_runtime_api::http::{Request, Response};
use aws_smithy_schema::http_protocol::HttpRpcProtocol;
use aws_smithy_schema::protocol::ClientProtocolInner;
use aws_smithy_schema::serde::{SerdeError, SerializableStruct, ShapeDeserializer};
use aws_smithy_schema::{shape_id, Schema, ShapeId};
use aws_smithy_types::config_bag::ConfigBag;

/// RPC v2 CBOR protocol (`smithy.protocols#rpcv2Cbor`).
#[derive(Debug)]
pub struct RpcV2CborProtocol {
    inner: HttpRpcProtocol<CborCodec>,
}

impl RpcV2CborProtocol {
    /// Creates a new RPC v2 CBOR protocol instance.
    pub fn new() -> Self {
        Self {
            inner: HttpRpcProtocol::new(
                shape_id!("smithy.protocols", "rpcv2Cbor"),
                CborCodec::new(CborCodecSettings::default()),
                "application/cbor",
            ),
        }
    }
}

impl Default for RpcV2CborProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientProtocolInner for RpcV2CborProtocol {
    type Request = Request;
    type Response = Response;

    fn protocol_id(&self) -> &ShapeId {
        self.inner.protocol_id()
    }

    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema,
        endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<Request, SerdeError> {
        self.inner
            .serialize_request(input, input_schema, endpoint, cfg)
    }

    fn deserialize_response<'a>(
        &self,
        response: &'a Response,
        output_schema: &Schema,
        cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
        self.inner
            .deserialize_response(response, output_schema, cfg)
    }

    fn payload_codec(&self) -> Option<&dyn aws_smithy_schema::codec::DynCodec> {
        self.inner.payload_codec()
    }

    fn update_endpoint(
        &self,
        request: &mut Request,
        endpoint: &aws_smithy_types::endpoint::Endpoint,
        cfg: &ConfigBag,
    ) -> Result<(), SerdeError> {
        self.inner.update_endpoint(request, endpoint, cfg)
    }
}

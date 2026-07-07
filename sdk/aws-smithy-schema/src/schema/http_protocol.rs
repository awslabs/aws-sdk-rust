/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP-based client protocol implementations.
//!
//! This module provides two concrete protocol types that implement
//! [`crate::schema::protocol::ClientProtocolInner`] for HTTP transports:
//!
//! - [`HttpBindingProtocol`] — for REST-style protocols (e.g., `restJson1`, `restXml`)
//!   that split members across HTTP headers, query strings, URI labels, and the payload.
//! - [`HttpRpcProtocol`] — for RPC-style protocols (e.g., `awsJson1_0`, `rpcv2Cbor`)
//!   that put everything in the body and ignore HTTP bindings.
//!
//! # Protocol hierarchy
//!
//! ```text
//! ClientProtocolInner (impl side)  →  ClientProtocol<Req, Res> (dyn side)
//!   ├─ HttpBindingProtocol<C>   (REST: restJson, restXml)
//!   └─ HttpRpcProtocol<C>       (RPC: awsJson, rpcv2Cbor)
//! ```
//!
//! Concrete protocol types like `AwsRestJsonProtocol` are thin wrappers that
//! construct one of these with the appropriate codec and settings.

mod binding;
mod rpc;

pub use binding::{percent_encode, HttpBindingProtocol};
pub use rpc::HttpRpcProtocol;

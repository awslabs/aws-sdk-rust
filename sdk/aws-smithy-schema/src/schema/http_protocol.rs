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
mod bound_value;
// Constructed by the REST protocols once they take ownership of response deserialization.
mod response;
mod rpc;

pub use binding::{percent_encode, HttpBindingProtocol};
pub use response::{http_error_deserializer, http_output_deserializer};
pub use rpc::HttpRpcProtocol;

/// Resolves the timestamp format a schema asks for, falling back to `default`.
///
/// The fallback is the location's protocol default, which differs by where the value sits
/// in the message: `http-date` in headers, `date-time` in query strings and URI labels.
/// Callers pass the default for their location rather than having it decided here, because
/// the same schema can be bound to different locations.
pub(crate) fn timestamp_format_or(
    schema: &crate::Schema<'_>,
    default: aws_smithy_types::date_time::Format,
) -> aws_smithy_types::date_time::Format {
    use aws_smithy_types::date_time::Format;
    match schema.timestamp_format() {
        Some(ts) => match ts.format() {
            crate::traits::TimestampFormat::EpochSeconds => Format::EpochSeconds,
            crate::traits::TimestampFormat::HttpDate => Format::HttpDate,
            crate::traits::TimestampFormat::DateTime => Format::DateTime,
        },
        None => default,
    }
}

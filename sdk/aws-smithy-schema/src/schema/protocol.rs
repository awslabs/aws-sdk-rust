/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Client protocol traits for protocol-agnostic request serialization and response deserialization.
//!
//! [`ClientProtocolInner`] is the trait implementors write. It carries associated
//! `Request` / `Response` types and allows transport-agnostic protocols (the SEP calls
//! this out as a requirement).
//!
//! [`ClientProtocol`] is the object-safe view that callers use through `dyn`. It's
//! parameterized over concrete request/response types (defaulted to HTTP) so
//! [`SharedClientProtocol`] can be stored in a [`ConfigBag`] and swapped at runtime.
//!
//! A blanket impl (`impl<P: ClientProtocolInner> ClientProtocol<P::Request, P::Response> for P`)
//! means implementors only write `ClientProtocolInner`; the object-safe view comes for
//! free. This mirrors the [`Codec`](crate::codec::Codec) / [`DynCodec`](crate::codec::DynCodec)
//! pair in the codec module — the same "static-dispatch inner trait + object-safe sibling"
//! pattern.
//!
//! # Implementing a custom protocol
//!
//! Third parties can create custom protocols and use them with any client without
//! modifying a code generator.
//!
//! ```ignore
//! use aws_smithy_schema::protocol::{apply_http_endpoint, ClientProtocolInner};
//! use aws_smithy_schema::{Schema, ShapeId};
//! use aws_smithy_schema::serde::SerializableStruct;
//!
//! #[derive(Debug)]
//! struct MyProtocol {
//!     codec: MyJsonCodec,
//! }
//!
//! impl ClientProtocolInner for MyProtocol {
//!     type Request = aws_smithy_runtime_api::http::Request;
//!     type Response = aws_smithy_runtime_api::http::Response;
//!
//!     fn protocol_id(&self) -> &ShapeId<'static> { &MY_PROTOCOL_ID }
//!
//!     fn serialize_request(
//!         &self,
//!         input: &dyn SerializableStruct,
//!         input_schema: &Schema<'_>,
//!         endpoint: &str,
//!         cfg: &ConfigBag,
//!     ) -> Result<Self::Request, SerdeError> {
//!         todo!()
//!     }
//!
//!     fn deserialize_response<'a>(
//!         &self,
//!         response: &'a Self::Response,
//!         output_schema: &Schema<'_>,
//!         cfg: &ConfigBag,
//!     ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
//!         todo!()
//!     }
//!
//!     fn update_endpoint(
//!         &self,
//!         request: &mut Self::Request,
//!         endpoint: &aws_smithy_types::endpoint::Endpoint,
//!         cfg: &ConfigBag,
//!     ) -> Result<(), SerdeError> {
//!         apply_http_endpoint(request, endpoint, cfg)
//!     }
//! }
//! ```

use crate::serde::{SerdeError, SerializableStruct, ShapeDeserializer};
use crate::{Schema, ShapeId};
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::endpoint::Endpoint;
use aws_smithy_types::error::metadata::{Builder as ErrorMetadataBuilder, ErrorMetadata};

/// Statically-dispatched client protocol trait — the one implementors write.
///
/// `Request` and `Response` are associated types so a protocol can target any transport
/// (HTTP, MQTT, Unix-socket, in-memory, …). For the common HTTP case, set both to
/// `aws_smithy_runtime_api::http::Request` / `Response`.
///
/// Callers who need to store a protocol behind `dyn` (e.g., in a [`ConfigBag`] for
/// runtime swapping) should use the object-safe [`ClientProtocol`] trait instead.
/// Every `ClientProtocolInner` is automatically a
/// `ClientProtocol<Self::Request, Self::Response>` via a blanket impl, so implementors
/// never write `ClientProtocol` manually.
///
/// See [`apply_http_endpoint`] for the canonical HTTP implementation of
/// `update_endpoint`.
///
/// # Lifecycle
///
/// Instances are immutable and thread-safe. They are typically created once and
/// shared across all requests for a client.
pub trait ClientProtocolInner: Send + Sync + std::fmt::Debug {
    /// The protocol's request message type (e.g., `http::Request`).
    type Request;

    /// The protocol's response message type (e.g., `http::Response`).
    type Response;

    /// Returns the Smithy shape ID of this protocol.
    fn protocol_id(&self) -> &ShapeId<'static>;

    /// Serializes an operation input into a request message.
    ///
    /// # The protocol owns its own wire format
    ///
    /// Anything this protocol puts on the wire that *the protocol alone determines* must be
    /// resolved here, not emitted by codegen. A client generated for one protocol can be pointed at
    /// another with `Config::builder().protocol(..)`, so anything codegen bakes in based on the
    /// generated protocol is wrong after a swap: left behind when this protocol is swapped out, and
    /// missing when it is swapped in. Concretely, an implementor owns:
    ///
    /// - **Framing headers.** rpcv2Cbor sets `smithy-protocol` and `accept`; awsJson sets
    ///   `X-Amz-Target`. Codegen's schema path deliberately emits none of these.
    /// - **The request path**, per the note on `endpoint` below.
    /// - **Model facts the protocol needs but a caller cannot know**, read from `cfg`:
    ///   [`ServiceShapeName`], [`ServiceVersion`], [`ServiceXmlNamespace`]. Generated clients store
    ///   these regardless of which protocol they were generated for, precisely so a swapped-in
    ///   protocol can find them. Keep an explicit builder as an override for what the model cannot
    ///   express -- a target prefix that is not the service shape name, for instance.
    ///
    /// Headers determined by the *service* rather than the protocol -- `x-amzn-query-mode`, from
    /// `@awsQueryCompatible` -- are outside this rule and stay in codegen, because their value does
    /// not change when the protocol does.
    ///
    /// # `endpoint` is advisory
    ///
    /// `endpoint` is a request **path** (or `""`), never a host; scheme and authority are merged
    /// later by [`apply_http_endpoint`]. It is whatever codegen computed *for the protocol the
    /// client was generated for*, so a protocol that alone determines its route must ignore it:
    ///
    /// - **Fixed route** -- awsJson and awsQuery are specified to `POST /`, so they pass `/` and
    ///   ignore the argument entirely. Forwarding it would let an rpcv2Cbor-generated client POST
    ///   to `/service/{service}/operation/{operation}`.
    /// - **Route derived from model facts** -- rpcv2Cbor computes
    ///   `/service/{service}/operation/{operation}` from `cfg`, falling back to `endpoint` only
    ///   when those facts are absent.
    /// - **Route from `@http` bindings** -- REST protocols expand the operation's `@http` template
    ///   from the schema, which is authoritative; `endpoint` is ignored. Generated REST clients
    ///   pass `""`, so this costs them nothing, and it stops an RPC route from being prefixed onto
    ///   the template after a swap. `endpoint` acts as the template only for a schema that carries
    ///   no `@http` trait at all.
    ///
    /// The shared [`HttpRpcProtocol`](crate::schema::http_protocol::HttpRpcProtocol) helper
    /// deliberately does *not* hard-code `/`; only a concrete protocol knows whether its route is
    /// constant.
    ///
    /// [`ServiceShapeName`]: crate::protocol::ServiceShapeName
    /// [`ServiceVersion`]: crate::protocol::ServiceVersion
    /// [`ServiceXmlNamespace`]: crate::protocol::ServiceXmlNamespace
    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema<'_>,
        endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<Self::Request, SerdeError>;

    /// Deserializes a response message, returning a boxed [`ShapeDeserializer`] over
    /// the response body.
    ///
    /// The deserializer reads only body members. Callers that also need to read
    /// transport-bound members (HTTP headers, status code) do that directly in
    /// generated code before consuming the deserializer.
    fn deserialize_response<'a>(
        &self,
        response: &'a Self::Response,
        output_schema: &Schema<'_>,
        cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError>;

    /// Extracts canonical error metadata (code, message, request id) from a
    /// response's wire envelope.
    ///
    /// Returns a [`Builder`](ErrorMetadataBuilder) so callers can attach
    /// per-request fields (e.g., `x-amzn-RequestId` from an HTTP header) before
    /// finalizing.
    ///
    /// Concrete protocols override this to extract their envelope-specific
    /// fields:
    /// - awsJson1.0 / awsJson1.1: `__type` from the body, `X-Amzn-Errortype`
    ///   header fallback.
    /// - restJson1: same as awsJson.
    /// - restXml (wrapped): `<ErrorResponse><Error><Code>` etc.
    /// - restXml (`@restXml(noErrorWrapping: true)`): `<Error><Code>` etc.
    /// - awsQuery / ec2Query: `<ErrorResponse><Error><Code>` etc.
    /// - rpcv2Cbor: `__type` from the CBOR map.
    ///
    /// The default implementation returns an empty
    /// [`Builder`](ErrorMetadataBuilder) — sufficient for protocols that
    /// haven't migrated to schema-driven error dispatch yet, but
    /// callers will see `Option::None` for `code()` / `message()` and treat
    /// the response as an unhandled error.
    fn parse_error_metadata(
        &self,
        response: &Self::Response,
        cfg: &ConfigBag,
    ) -> Result<ErrorMetadataBuilder, SerdeError> {
        let _ = (response, cfg);
        Ok(ErrorMetadata::builder())
    }

    /// Returns a [`ShapeDeserializer`] positioned at the body of an error
    /// response — *inside* the protocol's error envelope, where applicable.
    ///
    /// Generated error dispatch code calls this to obtain a deserializer
    /// usable with `<SpecificError>::deserialize_with_response(...)` (or the
    /// equivalent generated `deserialize`), regardless of which protocol is
    /// active at runtime.
    ///
    /// For envelope-less protocols (awsJson1.0/1.1, restJson1, rpcv2Cbor) the
    /// default implementation suffices: the body root *is* the error body, so
    /// it forwards to [`deserialize_response`](Self::deserialize_response)
    /// against [`prelude::DOCUMENT`](crate::prelude::DOCUMENT).
    ///
    /// Envelope-bearing protocols (restXml wrapped / unwrapped, awsQuery,
    /// ec2Query) MUST override to strip the outer `<ErrorResponse>` /
    /// `<Error>` wrapper before returning the deserializer.
    fn deserialize_error_response<'a>(
        &self,
        response: &'a Self::Response,
        cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
        self.deserialize_response(response, &crate::prelude::DOCUMENT, cfg)
    }

    /// Updates a previously serialized request with a resolved endpoint.
    ///
    /// Required by SEP requirement 7. The orchestrator calls this after endpoint
    /// resolution, which happens *after* `serialize_request`.
    ///
    /// HTTP protocols should implement this as:
    /// ```ignore
    /// apply_http_endpoint(request, endpoint, cfg)
    /// ```
    /// (See [`apply_http_endpoint`].) Non-HTTP protocols implement the transport's
    /// equivalent.
    fn update_endpoint(
        &self,
        request: &mut Self::Request,
        endpoint: &Endpoint,
        cfg: &ConfigBag,
    ) -> Result<(), SerdeError>;

    /// Returns the codec used for payload (de)serialization, if any.
    ///
    /// See [`DynCodec`](crate::codec::DynCodec) for why the codec is exposed
    /// through the object-safe sibling.
    fn payload_codec(&self) -> Option<&dyn crate::codec::DynCodec> {
        None
    }

    /// The media type used to label a **structured event-stream payload**, if this
    /// protocol supports event streams — for example `application/cbor` or
    /// `application/json`.
    ///
    /// Every event-stream frame carries a `:content-type` header describing its
    /// payload. The payload itself is encoded by whichever protocol is selected at
    /// runtime, so this label has to come from that same protocol; a label baked in
    /// when the client was generated contradicts the bytes as soon as a different
    /// protocol is selected, and a peer that honours the header then decodes with
    /// the wrong codec.
    ///
    /// # Why this is on the protocol and not on [`Codec`](crate::codec::Codec)
    ///
    /// The value is protocol-determined, not format-determined, and one codec
    /// serves several protocols that disagree about it. A single `JsonCodec` backs
    /// restJson1, awsJson1_0 and awsJson1_1, whose request content types are
    /// `application/json`, `application/x-amz-json-1.0` and
    /// `application/x-amz-json-1.1` respectively. A codec-level accessor could
    /// therefore only ever return one of those, and would describe event payloads
    /// correctly only because the three JSON protocols happen to agree *there*.
    ///
    /// # What this is not
    ///
    /// This is **not** the request `Content-Type`. For awsJson the two differ, per
    /// the above. It is also not the content type of a payload whose type is fixed
    /// by the *shape* rather than the format: an `@eventPayload` blob is
    /// `application/octet-stream` and a string is `text/plain` under every
    /// protocol, so those stay with the code generator and must not be sourced
    /// from here.
    ///
    /// Returns `None` by default, meaning the protocol declares no such media type
    /// — callers must keep a fallback. The default keeps this addition
    /// non-breaking for third-party protocols, which the SEP requires be able to
    /// exist without modifying a code generator.
    fn event_stream_media_type(&self) -> Option<&str> {
        None
    }

    /// Extracts canonical error metadata from the payload of an event-stream
    /// `exception` frame.
    ///
    /// This is the event-stream counterpart of
    /// [`parse_error_metadata`](Self::parse_error_metadata), and it exists
    /// separately for a typing reason rather than a behavioral one: that method
    /// takes `&Self::Response`, an HTTP response, and an event-stream frame is not
    /// one. The parsing itself is identical — both read a protocol-specific error
    /// envelope out of a byte payload — so implementors should delegate to the same
    /// helper they use there.
    ///
    /// # Why this must be resolved by the protocol
    ///
    /// The frame's payload is encoded by whichever protocol is selected at runtime
    /// (via [`payload_codec`](Self::payload_codec)), so its error envelope must be
    /// parsed by that same protocol. A code generator cannot decide this: it knows
    /// only the protocol the client was generated for, and after a runtime protocol
    /// swap that is the wrong one. Getting it wrong is not silent — the parse fails
    /// and the caller reports an unhandled error — but it costs the error code, and
    /// with it the modeled error variant and any retry classification keyed on that
    /// code.
    ///
    /// # Scope
    ///
    /// Takes the payload only. An event-stream frame carries no HTTP headers, so
    /// there is nothing to pass for the header-borne discriminators some protocols
    /// also accept (restJson1's `x-amzn-errortype`, awsQuery-compatible's
    /// `x-amzn-query-error`); implementors should parse as though the header map
    /// were empty. The frame's own `:exception-type` header is a separate mechanism
    /// and is handled by generated dispatch code before this is called.
    ///
    /// The default returns an empty
    /// [`Builder`](ErrorMetadataBuilder), matching
    /// [`parse_error_metadata`](Self::parse_error_metadata): callers see
    /// `Option::None` for `code()` / `message()` and treat the frame as an
    /// unhandled error. The default keeps this addition non-breaking for
    /// third-party protocols, which the SEP requires be able to exist without
    /// modifying a code generator.
    ///
    /// Implementors of a protocol without event streams should leave this alone.
    fn parse_event_stream_error_metadata(
        &self,
        payload: &[u8],
    ) -> Result<ErrorMetadataBuilder, SerdeError> {
        let _ = payload;
        Ok(ErrorMetadata::builder())
    }
}

/// Object-safe view of [`ClientProtocolInner`] parameterized over concrete
/// request / response types.
///
/// This is what callers hold behind `dyn`, for example,
/// [`SharedClientProtocol`] stores `Arc<dyn ClientProtocol<Req, Res>>` so the
/// protocol can be swapped at runtime. The generic `Req` / `Res` parameters
/// default to HTTP so existing call sites remain source-compatible.
///
/// Every `ClientProtocolInner` gets `ClientProtocol` for free via a blanket
/// impl; implementors should write `ClientProtocolInner` only.
pub trait ClientProtocol<
    Req = aws_smithy_runtime_api::http::Request,
    Res = aws_smithy_runtime_api::http::Response,
>: Send + Sync + std::fmt::Debug
{
    /// Returns the Smithy shape ID of this protocol.
    fn protocol_id(&self) -> &ShapeId<'static>;

    /// Serializes an operation input into a request message.
    ///
    /// See [`ClientProtocolInner::serialize_request`] for the invariant an implementor must uphold:
    /// the protocol owns its own framing headers and request path, and `endpoint` is advisory.
    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema<'_>,
        endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<Req, SerdeError>;

    /// Deserializes a response message, returning a boxed [`ShapeDeserializer`].
    fn deserialize_response<'a>(
        &self,
        response: &'a Res,
        output_schema: &Schema<'_>,
        cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError>;

    /// Extracts canonical error metadata from a response.
    ///
    /// See [`ClientProtocolInner::parse_error_metadata`] for the contract.
    fn parse_error_metadata(
        &self,
        response: &Res,
        cfg: &ConfigBag,
    ) -> Result<ErrorMetadataBuilder, SerdeError>;

    /// Returns a [`ShapeDeserializer`] positioned at the body of an error
    /// response — inside the protocol's error envelope, where applicable.
    ///
    /// See [`ClientProtocolInner::deserialize_error_response`] for the contract.
    fn deserialize_error_response<'a>(
        &self,
        response: &'a Res,
        cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError>;

    /// Updates a previously serialized request with a resolved endpoint.
    fn update_endpoint(
        &self,
        request: &mut Req,
        endpoint: &Endpoint,
        cfg: &ConfigBag,
    ) -> Result<(), SerdeError>;

    /// Returns the codec used for payload (de)serialization, if any.
    fn payload_codec(&self) -> Option<&dyn crate::codec::DynCodec>;

    /// The media type used to label a structured event-stream payload, if any.
    ///
    /// See [`ClientProtocolInner::event_stream_media_type`] for why this is a
    /// protocol-level fact rather than a codec-level one.
    fn event_stream_media_type(&self) -> Option<&str>;

    /// Extracts canonical error metadata from an event-stream `exception` frame's
    /// payload.
    ///
    /// See [`ClientProtocolInner::parse_event_stream_error_metadata`] for the
    /// contract and for why the payload's envelope must be parsed by the protocol
    /// selected at runtime.
    fn parse_event_stream_error_metadata(
        &self,
        payload: &[u8],
    ) -> Result<ErrorMetadataBuilder, SerdeError>;
}

// Blanket impl: any `ClientProtocolInner` is automatically a `ClientProtocol`
// parameterized over its associated `Request` / `Response` types.
impl<P> ClientProtocol<P::Request, P::Response> for P
where
    P: ClientProtocolInner,
{
    fn protocol_id(&self) -> &ShapeId<'static> {
        <Self as ClientProtocolInner>::protocol_id(self)
    }

    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema<'_>,
        endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<P::Request, SerdeError> {
        <Self as ClientProtocolInner>::serialize_request(self, input, input_schema, endpoint, cfg)
    }

    fn deserialize_response<'a>(
        &self,
        response: &'a P::Response,
        output_schema: &Schema<'_>,
        cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
        <Self as ClientProtocolInner>::deserialize_response(self, response, output_schema, cfg)
    }

    fn parse_error_metadata(
        &self,
        response: &P::Response,
        cfg: &ConfigBag,
    ) -> Result<ErrorMetadataBuilder, SerdeError> {
        <Self as ClientProtocolInner>::parse_error_metadata(self, response, cfg)
    }

    fn deserialize_error_response<'a>(
        &self,
        response: &'a P::Response,
        cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
        <Self as ClientProtocolInner>::deserialize_error_response(self, response, cfg)
    }

    fn update_endpoint(
        &self,
        request: &mut P::Request,
        endpoint: &Endpoint,
        cfg: &ConfigBag,
    ) -> Result<(), SerdeError> {
        <Self as ClientProtocolInner>::update_endpoint(self, request, endpoint, cfg)
    }

    fn payload_codec(&self) -> Option<&dyn crate::codec::DynCodec> {
        <Self as ClientProtocolInner>::payload_codec(self)
    }

    fn event_stream_media_type(&self) -> Option<&str> {
        <Self as ClientProtocolInner>::event_stream_media_type(self)
    }

    fn parse_event_stream_error_metadata(
        &self,
        payload: &[u8],
    ) -> Result<ErrorMetadataBuilder, SerdeError> {
        <Self as ClientProtocolInner>::parse_event_stream_error_metadata(self, payload)
    }
}

/// Applies a resolved endpoint to an HTTP request.
///
/// This is the canonical HTTP implementation of
/// [`ClientProtocolInner::update_endpoint`]. HTTP protocols should delegate to it.
///
/// Handles endpoint prefixes (for `EndpointPrefix`-enabled operations) and
/// endpoint-supplied headers.
pub fn apply_http_endpoint(
    request: &mut aws_smithy_runtime_api::http::Request,
    endpoint: &Endpoint,
    cfg: &ConfigBag,
) -> Result<(), SerdeError> {
    use std::borrow::Cow;

    let endpoint_prefix = cfg.load::<aws_smithy_runtime_api::client::endpoint::EndpointPrefix>();
    let endpoint_url = match endpoint_prefix {
        None => Cow::Borrowed(endpoint.url()),
        Some(prefix) => {
            let parsed: http::Uri = endpoint
                .url()
                .parse()
                .map_err(|e| SerdeError::custom(format!("invalid endpoint URI: {e}")))?;
            let scheme = parsed.scheme_str().unwrap_or_default();
            let prefix = prefix.as_str();
            let authority = parsed.authority().map(|a| a.as_str()).unwrap_or_default();
            let path_and_query = parsed
                .path_and_query()
                .map(|pq| pq.as_str())
                .unwrap_or_default();
            Cow::Owned(format!("{scheme}://{prefix}{authority}{path_and_query}"))
        }
    };

    request.uri_mut().set_endpoint(&endpoint_url).map_err(|e| {
        SerdeError::custom(format!("failed to apply endpoint `{endpoint_url}`: {e}"))
    })?;

    for (header_name, header_values) in endpoint.headers() {
        request.headers_mut().remove(header_name);
        for value in header_values {
            request
                .headers_mut()
                .append(header_name.to_owned(), value.to_owned());
        }
    }

    Ok(())
}

/// The name of the Smithy `service` shape a client was generated for.
///
/// Some protocols derive parts of the wire format from model names rather than
/// from HTTP binding traits. RPC v2 CBOR is the canonical example: every request
/// is routed to `/service/{serviceName}/operation/{operationName}`, where
/// `serviceName` is the *service shape name* — not the `@aws.api#service`
/// `sdkId`, and not the shape's namespace.
///
/// Because [`SharedClientProtocol`] can be swapped at runtime, a protocol cannot
/// rely on codegen having baked its route into the generated request path: a
/// client generated for `awsJson1_0` may have `RpcV2CborProtocol` plugged in via
/// `Config::builder().protocol(..)`. Generated clients therefore store this entry
/// in the config bag regardless of which protocol they were generated for, so
/// whichever protocol ends up being used can resolve the names it needs. The
/// companion operation name comes from
/// [`Metadata::name`](aws_smithy_runtime_api::client::orchestrator::Metadata::name).
///
/// See <https://github.com/smithy-lang/smithy-rs/issues/4801>.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ServiceShapeName(std::borrow::Cow<'static, str>);

impl ServiceShapeName {
    /// Creates a new [`ServiceShapeName`] from the Smithy service shape name.
    ///
    /// Accepts a codegen-emitted `&'static str` as well as a `String`
    /// materialized at runtime from a parsed model.
    pub fn new(name: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(name.into())
    }

    /// Returns the service shape name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl aws_smithy_types::config_bag::Storable for ServiceShapeName {
    type Storer = aws_smithy_types::config_bag::StoreReplace<Self>;
}

/// The namespace of the Smithy `service` shape a client was generated for.
///
/// This is the `com.amazonaws.dynamodb` in `com.amazonaws.dynamodb#DynamoDB_20120810`.
/// Together with [`ServiceShapeName`] it forms the service's full shape ID; the two are
/// separate entries rather than one because [`ConfigBag`] is keyed by type, so each protocol
/// loads exactly the facts it needs and new facts stay additive.
///
/// **Not to be confused with [`ServiceXmlNamespace`]**, despite the shared word. That one is
/// the `@xmlNamespace` *trait* — a URI restXml applies as the default `xmlns` on root
/// elements — and neither value is derivable from the other. CloudWatch Logs is the clearest
/// illustration: its shape-ID namespace is `com.amazonaws.cloudwatchlogs` while its
/// `@xmlNamespace` URI is `http://monitoring.amazonaws.com/doc/2014-03-28/`. The trait is
/// also optional, carried by roughly half of AWS service shapes, whereas every shape ID has
/// a namespace by construction — which is why this entry is stored unconditionally and
/// `ServiceXmlNamespace` is not.
///
/// Protocols use this as the *default namespace* when resolving a document type's shape
/// discriminator. Some services serialize a discriminator as a bare shape name rather than an
/// absolute shape ID — a `__type` of `Widget` instead of `com.example#Widget` — and the
/// receiving client is expected to qualify it with the service's namespace. Without it a
/// relative discriminator cannot be resolved to a registered type at all.
///
/// Stored for the same reason as [`ServiceShapeName`]: it is knowable only from the model, and
/// a customer may select a different protocol at runtime via `Config::builder().protocol(..)`
/// on a client generated for some other protocol. Baking it into a constructor call at codegen
/// time means a swapped-in protocol either gets no value or gets one the caller had to know to
/// supply. Generated clients therefore store it regardless of which protocol they were
/// generated for.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ServiceShapeNamespace(std::borrow::Cow<'static, str>);

impl ServiceShapeNamespace {
    /// Creates a new [`ServiceShapeNamespace`] from the Smithy service shape's namespace.
    ///
    /// Accepts a codegen-emitted `&'static str` as well as a `String` materialized at runtime
    /// from a parsed model.
    pub fn new(namespace: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(namespace.into())
    }

    /// Returns the service shape's namespace.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl aws_smithy_types::config_bag::Storable for ServiceShapeNamespace {
    type Storer = aws_smithy_types::config_bag::StoreReplace<Self>;
}

/// The Smithy service shape's `version`, stored in a [`ConfigBag`] by generated clients.
///
/// awsQuery puts this on the wire as the `Version=` form parameter, so it is a request-shaping
/// fact that only the model knows. Stored for the same reason as [`ServiceShapeName`]: a customer
/// can select awsQuery via `Config::builder().protocol(..)` on a client generated for some other
/// protocol, and could not otherwise supply the right value.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ServiceVersion(std::borrow::Cow<'static, str>);

impl ServiceVersion {
    /// Creates a new [`ServiceVersion`].
    ///
    /// Accepts a codegen-emitted `&'static str` as well as a `String` materialized at runtime
    /// from a parsed model.
    pub fn new(version: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(version.into())
    }

    /// Returns the service version.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl aws_smithy_types::config_bag::Storable for ServiceVersion {
    type Storer = aws_smithy_types::config_bag::StoreReplace<Self>;
}

/// The service-level `@xmlNamespace` trait, stored in a [`ConfigBag`] by generated clients.
///
/// restXml applies this as the default `xmlns` on request and response root elements, so it is a
/// request-shaping fact that only the model knows. Stored for the same reason as
/// [`ServiceShapeName`].
///
/// `@xmlNamespace` is a prelude trait rather than a restXml-specific one, so it is resolvable from
/// any model — unlike `@restXml(noErrorWrapping)`, which a non-restXml model simply does not carry
/// and which therefore stays caller-supplied.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ServiceXmlNamespace {
    uri: std::borrow::Cow<'static, str>,
    prefix: Option<std::borrow::Cow<'static, str>>,
}

impl ServiceXmlNamespace {
    /// Creates a new [`ServiceXmlNamespace`] from the trait's URI and optional prefix.
    pub fn new(
        uri: impl Into<std::borrow::Cow<'static, str>>,
        prefix: Option<std::borrow::Cow<'static, str>>,
    ) -> Self {
        Self {
            uri: uri.into(),
            prefix,
        }
    }

    /// Returns the namespace URI.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Returns the namespace prefix, if the trait declared one.
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}

impl aws_smithy_types::config_bag::Storable for ServiceXmlNamespace {
    type Storer = aws_smithy_types::config_bag::StoreReplace<Self>;
}

/// A shared, type-erased client protocol stored in a [`ConfigBag`].
///
/// Wraps `Arc<dyn ClientProtocol<Req, Res>>` so a protocol can be stored and
/// retrieved from the config bag for runtime protocol selection.
///
/// Defaults to HTTP transport types. Custom transports would use
/// `SharedClientProtocol<MyReq, MyRes>` and would need their own `Storable`
/// adaptation (not provided here — today only HTTP has a `Storable` impl,
/// reflecting the fact that the orchestrator is HTTP-concrete).
#[derive(Debug)]
pub struct SharedClientProtocol<
    Req = aws_smithy_runtime_api::http::Request,
    Res = aws_smithy_runtime_api::http::Response,
> {
    inner: std::sync::Arc<dyn ClientProtocol<Req, Res>>,
}

// Manual `Clone` — `Arc` is cheaply cloneable regardless of whether the inner
// `Req` / `Res` types are themselves `Clone`, so this impl avoids a spurious
// `Req: Clone, Res: Clone` bound that `#[derive(Clone)]` would introduce.
impl<Req, Res> Clone for SharedClientProtocol<Req, Res> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Req, Res> SharedClientProtocol<Req, Res>
where
    Req: 'static,
    Res: 'static,
{
    /// Creates a new shared protocol from any [`ClientProtocol<Req, Res>`] impl.
    ///
    /// In practice callers pass a concrete type that implements
    /// [`ClientProtocolInner`] — the blanket `impl<P: ClientProtocolInner>
    /// ClientProtocol<P::Request, P::Response> for P` makes every
    /// `ClientProtocolInner` automatically usable here.
    pub fn new<P>(protocol: P) -> Self
    where
        P: ClientProtocol<Req, Res> + 'static,
    {
        Self {
            inner: std::sync::Arc::new(protocol),
        }
    }
}

impl<Req, Res> std::ops::Deref for SharedClientProtocol<Req, Res> {
    type Target = dyn ClientProtocol<Req, Res>;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

// Only the HTTP specialization is storable in the config bag, matching the
// orchestrator's HTTP-concrete wiring today. This is paired with the three
// `protocol(…)` setters — `aws_types::SdkConfig::Builder::protocol`,
// `aws_config::ConfigLoader::protocol`, and the generated
// `ConfigBuilder::protocol` — all of which accept `impl ClientProtocol +
// 'static` (resolving via defaults to the HTTP specialization) and store
// the resulting `SharedClientProtocol<http::Request, http::Response>` here.
//
// Non-HTTP transports would add their own Storable newtype alongside their
// transport integration (with its own dedicated setter) rather than
// generalizing this impl — see §10.2 of the implementation overview.
impl aws_smithy_types::config_bag::Storable
    for SharedClientProtocol<
        aws_smithy_runtime_api::http::Request,
        aws_smithy_runtime_api::http::Response,
    >
{
    type Storer = aws_smithy_types::config_bag::StoreReplace<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serde::{SerdeError, SerializableStruct, ShapeDeserializer};
    use crate::{Schema, ShapeId};
    use aws_smithy_runtime_api::http::{Request, Response, StatusCode};
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_types::config_bag::{ConfigBag, Layer};
    use aws_smithy_types::endpoint::Endpoint;

    /// Minimal protocol impl that uses the HTTP apply_http_endpoint helper.
    #[derive(Debug)]
    struct StubProtocol;

    static STUB_ID: ShapeId<'static> =
        ShapeId::from_parts("test#StubProtocol", "test", "StubProtocol");

    impl ClientProtocolInner for StubProtocol {
        type Request = Request;
        type Response = Response;

        fn protocol_id(&self) -> &ShapeId<'static> {
            &STUB_ID
        }
        fn serialize_request(
            &self,
            _input: &dyn SerializableStruct,
            _input_schema: &Schema<'_>,
            _endpoint: &str,
            _cfg: &ConfigBag,
        ) -> Result<Request, SerdeError> {
            unimplemented!()
        }
        fn deserialize_response<'a>(
            &self,
            _response: &'a Response,
            _output_schema: &Schema<'_>,
            _cfg: &ConfigBag,
        ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
            unimplemented!()
        }
        fn update_endpoint(
            &self,
            request: &mut Request,
            endpoint: &Endpoint,
            cfg: &ConfigBag,
        ) -> Result<(), SerdeError> {
            apply_http_endpoint(request, endpoint, cfg)
        }
    }

    fn request_with_uri(uri: &str) -> Request {
        let mut req = Request::new(SdkBody::empty());
        req.set_uri(uri).unwrap();
        req
    }

    #[test]
    fn basic_endpoint() {
        let proto = StubProtocol;
        let mut req = request_with_uri("/original/path");
        let endpoint = Endpoint::builder()
            .url("https://service.us-east-1.amazonaws.com")
            .build();
        let cfg = ConfigBag::base();

        ClientProtocolInner::update_endpoint(&proto, &mut req, &endpoint, &cfg).unwrap();
        assert_eq!(
            req.uri(),
            "https://service.us-east-1.amazonaws.com/original/path"
        );
    }

    #[test]
    fn endpoint_with_prefix() {
        let proto = StubProtocol;
        let mut req = request_with_uri("/path");
        let endpoint = Endpoint::builder()
            .url("https://service.us-east-1.amazonaws.com")
            .build();
        let mut cfg = ConfigBag::base();
        let mut layer = Layer::new("test");
        layer.store_put(
            aws_smithy_runtime_api::client::endpoint::EndpointPrefix::new("myprefix.").unwrap(),
        );
        cfg.push_shared_layer(layer.freeze());

        ClientProtocolInner::update_endpoint(&proto, &mut req, &endpoint, &cfg).unwrap();
        assert_eq!(
            req.uri(),
            "https://myprefix.service.us-east-1.amazonaws.com/path"
        );
    }

    #[test]
    fn endpoint_with_headers() {
        let proto = StubProtocol;
        let mut req = request_with_uri("/path");
        let endpoint = Endpoint::builder()
            .url("https://example.com")
            .header("x-custom", "value1")
            .header("x-custom", "value2")
            .build();
        let cfg = ConfigBag::base();

        ClientProtocolInner::update_endpoint(&proto, &mut req, &endpoint, &cfg).unwrap();
        assert_eq!(req.uri(), "https://example.com/path");
        let values: Vec<&str> = req.headers().get_all("x-custom").collect();
        assert_eq!(values, vec!["value1", "value2"]);
    }

    #[test]
    fn endpoint_with_path() {
        let proto = StubProtocol;
        let mut req = request_with_uri("/operation");
        let endpoint = Endpoint::builder().url("https://example.com/base").build();
        let cfg = ConfigBag::base();

        ClientProtocolInner::update_endpoint(&proto, &mut req, &endpoint, &cfg).unwrap();
        assert_eq!(req.uri(), "https://example.com/base/operation");
    }

    // -- Default impls for parse_error_metadata + deserialize_error_response --

    #[test]
    fn parse_error_metadata_default_returns_empty_builder() {
        let proto = StubProtocol;
        let response = Response::new(StatusCode::try_from(500).unwrap(), SdkBody::empty());
        let cfg = ConfigBag::base();

        let builder = ClientProtocolInner::parse_error_metadata(&proto, &response, &cfg).unwrap();
        let meta = builder.build();
        assert!(meta.code().is_none());
        assert!(meta.message().is_none());
    }

    /// Records the [`Schema`] id passed to `deserialize_response` so the
    /// `deserialize_error_response` default forwarding can be asserted.
    /// Captures the FQN as a `String` so the fixture isn't tied to the
    /// schema's data lifetime.
    #[derive(Debug, Default)]
    struct RecordingProtocol {
        last_schema_id: std::sync::Mutex<Option<String>>,
    }

    static REC_ID: ShapeId<'static> =
        ShapeId::from_parts("test#RecordingProtocol", "test", "RecordingProtocol");

    impl ClientProtocolInner for RecordingProtocol {
        type Request = Request;
        type Response = Response;

        fn protocol_id(&self) -> &ShapeId<'static> {
            &REC_ID
        }
        fn serialize_request(
            &self,
            _input: &dyn SerializableStruct,
            _input_schema: &Schema<'_>,
            _endpoint: &str,
            _cfg: &ConfigBag,
        ) -> Result<Request, SerdeError> {
            unimplemented!()
        }
        fn deserialize_response<'a>(
            &self,
            _response: &'a Response,
            output_schema: &Schema<'_>,
            _cfg: &ConfigBag,
        ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
            *self
                .last_schema_id
                .lock()
                .expect("RecordingProtocol mutex poisoned") =
                Some(output_schema.shape_id().as_str().to_owned());
            // Return an Err so we don't have to construct a real deserializer;
            // the test only cares which schema was forwarded.
            Err(SerdeError::custom("recording stub"))
        }
        fn update_endpoint(
            &self,
            _request: &mut Request,
            _endpoint: &Endpoint,
            _cfg: &ConfigBag,
        ) -> Result<(), SerdeError> {
            unimplemented!()
        }
    }

    #[test]
    fn deserialize_error_response_default_forwards_with_prelude_document_schema() {
        let proto = RecordingProtocol::default();
        let response = Response::new(StatusCode::try_from(500).unwrap(), SdkBody::empty());
        let cfg = ConfigBag::base();

        // The default impl forwards to deserialize_response. Our recording
        // stub captures the schema and then returns an error — we don't
        // care about the result, only the schema observed.
        let _ = ClientProtocolInner::deserialize_error_response(&proto, &response, &cfg);

        let observed = proto
            .last_schema_id
            .lock()
            .expect("RecordingProtocol mutex poisoned")
            .clone()
            .expect("schema id was captured");
        assert_eq!(observed, crate::prelude::DOCUMENT.shape_id().as_str());
    }
}

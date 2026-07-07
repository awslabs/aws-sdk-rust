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
//!     fn protocol_id(&self) -> &ShapeId { &MY_PROTOCOL_ID }
//!
//!     fn serialize_request(
//!         &self,
//!         input: &dyn SerializableStruct,
//!         input_schema: &Schema,
//!         endpoint: &str,
//!         cfg: &ConfigBag,
//!     ) -> Result<Self::Request, SerdeError> {
//!         todo!()
//!     }
//!
//!     fn deserialize_response<'a>(
//!         &self,
//!         response: &'a Self::Response,
//!         output_schema: &Schema,
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
    fn protocol_id(&self) -> &ShapeId;

    /// Serializes an operation input into a request message.
    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema,
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
        output_schema: &Schema,
        cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError>;

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
}

/// Object-safe view of [`ClientProtocolInner`] parameterized over concrete
/// request / response types.
///
/// This is what callers hold behind `dyn` — for example,
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
    fn protocol_id(&self) -> &ShapeId;

    /// Serializes an operation input into a request message.
    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema,
        endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<Req, SerdeError>;

    /// Deserializes a response message, returning a boxed [`ShapeDeserializer`].
    fn deserialize_response<'a>(
        &self,
        response: &'a Res,
        output_schema: &Schema,
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
}

// Blanket impl: any `ClientProtocolInner` is automatically a `ClientProtocol`
// parameterized over its associated `Request` / `Response` types.
impl<P> ClientProtocol<P::Request, P::Response> for P
where
    P: ClientProtocolInner,
{
    fn protocol_id(&self) -> &ShapeId {
        <Self as ClientProtocolInner>::protocol_id(self)
    }

    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema,
        endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<P::Request, SerdeError> {
        <Self as ClientProtocolInner>::serialize_request(self, input, input_schema, endpoint, cfg)
    }

    fn deserialize_response<'a>(
        &self,
        response: &'a P::Response,
        output_schema: &Schema,
        cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
        <Self as ClientProtocolInner>::deserialize_response(self, response, output_schema, cfg)
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
    use aws_smithy_runtime_api::http::{Request, Response};
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_types::config_bag::{ConfigBag, Layer};
    use aws_smithy_types::endpoint::Endpoint;

    /// Minimal protocol impl that uses the HTTP apply_http_endpoint helper.
    #[derive(Debug)]
    struct StubProtocol;

    static STUB_ID: ShapeId = ShapeId::from_static("test#StubProtocol", "test", "StubProtocol");

    impl ClientProtocolInner for StubProtocol {
        type Request = Request;
        type Response = Response;

        fn protocol_id(&self) -> &ShapeId {
            &STUB_ID
        }
        fn serialize_request(
            &self,
            _input: &dyn SerializableStruct,
            _input_schema: &Schema,
            _endpoint: &str,
            _cfg: &ConfigBag,
        ) -> Result<Request, SerdeError> {
            unimplemented!()
        }
        fn deserialize_response<'a>(
            &self,
            _response: &'a Response,
            _output_schema: &Schema,
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
}

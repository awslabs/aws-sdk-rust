/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Typed runtime representations of Smithy serialization traits.
//!
//! These types allow type-safe downcasting from `&dyn Trait` via `as_any()`,
//! enabling protocol implementations to read trait values without string-matching
//! on Shape IDs.

use crate::{ShapeId, Trait};
use std::any::Any;

macro_rules! annotation_trait {
    ($(#[$meta:meta])* $name:ident, $ns:literal, $trait_name:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        #[allow(dead_code)] // Used by generated schema code
        pub struct $name;

        impl $name {
            /// The Shape ID for this trait.
            pub const TRAIT_ID: ShapeId<'static> = crate::shape_id!($ns, $trait_name);
        }

        impl Trait for $name {
            fn trait_id(&self) -> &ShapeId<'static> { &Self::TRAIT_ID }
            fn as_any(&self) -> &dyn Any { self }
        }
    };
}

macro_rules! string_trait {
    ($(#[$meta:meta])* $name:ident, $ns:literal, $trait_name:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        #[allow(dead_code)] // Used by generated schema code
        pub struct $name<'a> {
            value: &'a str,
        }

        #[allow(dead_code)] // Used by generated schema code
        impl<'a> $name<'a> {
            /// The Shape ID for this trait.
            pub const TRAIT_ID: ShapeId<'static> = crate::shape_id!($ns, $trait_name);

            /// Creates a new instance.
            ///
            /// `value` is borrowed for `'a`. For codegen-emitted schemas this is
            /// a string literal, so `'a` is `'static`; schemas materialized at
            /// runtime borrow from storage the caller owns, such as an arena
            /// holding the parsed model text.
            pub const fn new(value: &'a str) -> Self {
                Self { value }
            }

            /// Returns the trait value.
            ///
            /// Bound to the schema's data lifetime `'a`, not to `&self`, so the
            /// value can be stored anywhere the schema's data is valid. For a
            /// `'static` schema this is `&'static str`.
            pub fn value(&self) -> &'a str {
                self.value
            }
        }

        // Implemented only for the `'static` instantiation: `Trait` requires the
        // `Any` supertrait, which requires `Self: 'static`. Runtime-lifetime
        // wrappers are still readable through the typed accessors on `Schema`;
        // only the `dyn Trait` fallback map is `'static`-only. See
        // `trait_type.rs` for the `Any` constraint.
        impl Trait for $name<'static> {
            fn trait_id(&self) -> &ShapeId<'static> { &Self::TRAIT_ID }
            fn as_any(&self) -> &dyn Any { self }
        }
    };
}

// --- Serialization & Protocol traits ---

string_trait!(
    /// The `@jsonName` trait — overrides the JSON key for a member.
    JsonNameTrait,
    "smithy.api", "jsonName"
);

string_trait!(
    /// The `@xmlName` trait — overrides the XML element name.
    XmlNameTrait,
    "smithy.api", "xmlName"
);

string_trait!(
    /// The `@mediaType` trait — specifies the media type of a blob/string.
    MediaTypeTrait,
    "smithy.api", "mediaType"
);

annotation_trait!(
    /// The `@xmlAttribute` trait — serializes a member as an XML attribute.
    XmlAttributeTrait,
    "smithy.api", "xmlAttribute"
);

annotation_trait!(
    /// The `@xmlFlattened` trait — removes the wrapping element for lists/maps in XML.
    XmlFlattenedTrait,
    "smithy.api", "xmlFlattened"
);

// xmlNamespace is a structured trait carrying a URI and an optional prefix.
// Hand-written rather than generated via the `string_trait!` macro because the
// macro only models a single string value.

/// The `@xmlNamespace` trait — adds an XML namespace declaration to elements
/// generated for the targeted shape.
///
/// REST XML emits this as `xmlns="uri"` (no prefix) or `xmlns:prefix="uri"`
/// on the start tag of the element to which the trait applies.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used by generated schema code
pub struct XmlNamespaceTrait<'a> {
    uri: &'a str,
    prefix: Option<&'a str>,
}

#[allow(dead_code)] // Used by generated schema code
impl<'a> XmlNamespaceTrait<'a> {
    /// The Shape ID for this trait.
    pub const TRAIT_ID: ShapeId<'static> = crate::shape_id!("smithy.api", "xmlNamespace");

    /// Creates a new `XmlNamespaceTrait`.
    ///
    /// `uri` and `prefix` are borrowed for `'a`; see
    /// [`JsonNameTrait::new`](crate::traits::JsonNameTrait::new) for what that
    /// means for codegen-emitted versus runtime-materialized schemas.
    pub const fn new(uri: &'a str, prefix: Option<&'a str>) -> Self {
        Self { uri, prefix }
    }

    /// The namespace URI.
    pub fn uri(&self) -> &'a str {
        self.uri
    }

    /// The optional namespace prefix.
    ///
    /// When `Some(prefix)`, the namespace is declared as
    /// `xmlns:prefix="uri"`. When `None`, it is declared as `xmlns="uri"`.
    pub fn prefix(&self) -> Option<&'a str> {
        self.prefix
    }
}

impl Trait for XmlNamespaceTrait<'static> {
    fn trait_id(&self) -> &ShapeId<'static> {
        &Self::TRAIT_ID
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// --- Timestamp ---

/// The `@timestampFormat` trait — specifies the serialization format for timestamps.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Used by generated schema code
pub struct TimestampFormatTrait {
    format: TimestampFormat,
}

/// Timestamp serialization formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampFormat {
    /// Epoch seconds (e.g. `1515531081.123`).
    EpochSeconds,
    /// RFC 3339 date-time (e.g. `2018-01-09T18:47:00Z`).
    DateTime,
    /// RFC 7231 HTTP date (e.g. `Tue, 09 Jan 2018 18:47:00 GMT`).
    HttpDate,
}

#[allow(dead_code)] // Used by generated schema code
impl TimestampFormatTrait {
    /// The Shape ID for this trait.
    pub const TRAIT_ID: ShapeId<'static> = crate::shape_id!("smithy.api", "timestampFormat");

    /// Creates a new instance.
    pub const fn new(format: TimestampFormat) -> Self {
        Self { format }
    }

    /// Returns the timestamp format.
    pub fn format(&self) -> TimestampFormat {
        self.format
    }
}

impl Trait for TimestampFormatTrait {
    fn trait_id(&self) -> &ShapeId<'static> {
        &Self::TRAIT_ID
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// --- HTTP binding traits ---

// `@httpHeader` is hand-written rather than generated via `string_trait!`
// because its value has two cases. Header names are inserted into `Headers`,
// whose only allocation-free `AsHeaderComponent` impl is for `&'static str`
// (`aws-smithy-runtime-api`'s `MaybeStatic = Cow<'static, str>`), so the binder
// needs to recover a `'static` name from a schema whose lifetime is arbitrary.
// Modelling that as a two-arm enum rather than pinning the type means relaxing
// this trait later — to let runtime-materialized schemas borrow header names
// from a model arena — is a purely additive change: add a `new_borrowed`
// constructor and `value_static` starts returning `None`, with no signature
// changed.
//
// Both arms are references, so the type has no drop glue and stays
// `const fn`-constructible. That rules out `Cow<'static, str>` here: a
// `const fn` body cannot run destructors, and generated schemas are
// const-initialized statics.
#[derive(Debug, Clone, Copy)]
enum HeaderName<'a> {
    /// A codegen-emitted literal, or a runtime name interned/leaked to `'static`.
    Static(&'static str),
    /// Reserved for runtime-materialized schemas that borrow the header name
    /// from storage the caller owns. Not publicly constructible yet; adding a
    /// constructor for it is backwards compatible.
    #[allow(dead_code)]
    Borrowed(&'a str),
}

/// The `@httpHeader` trait — binds a member to an HTTP header.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used by generated schema code
pub struct HttpHeaderTrait<'a> {
    value: HeaderName<'a>,
}

#[allow(dead_code)] // Used by generated schema code
impl<'a> HttpHeaderTrait<'a> {
    /// The Shape ID for this trait.
    pub const TRAIT_ID: ShapeId<'static> = crate::shape_id!("smithy.api", "httpHeader");

    /// Creates a new instance from a `'static` header name.
    ///
    /// Unlike the other wire-format traits this requires `'static`, so that
    /// [`value_static`](Self::value_static) can hand the HTTP binder a name it
    /// inserts into `Headers` without allocating. Schemas materialized at
    /// runtime must intern or leak their header names; every other
    /// wire-format trait accepts an arbitrary lifetime.
    pub const fn new(value: &'static str) -> Self {
        Self {
            value: HeaderName::Static(value),
        }
    }

    /// Returns the header name.
    ///
    /// Bound to the schema's data lifetime `'a`, matching the other trait
    /// wrappers. Use [`value_static`](Self::value_static) when the name has to
    /// outlive the schema, as it does for header insertion.
    pub fn value(&self) -> &'a str {
        match self.value {
            HeaderName::Static(v) => v,
            HeaderName::Borrowed(v) => v,
        }
    }

    /// Returns the header name if it is `'static`.
    ///
    /// This is what `Headers::insert` needs for an allocation-free insert.
    ///
    /// Always `Some` today, because [`new`](Self::new) is the only constructor
    /// and it requires `'static`. Callers must nonetheless handle `None` and
    /// fall back to an owned string rather than unwrapping: relaxing this
    /// trait to accept runtime-borrowed names is intended to be additive, and
    /// at that point this returns `None` for arena-borrowed names.
    pub fn value_static(&self) -> Option<&'static str> {
        match self.value {
            HeaderName::Static(v) => Some(v),
            HeaderName::Borrowed(_) => None,
        }
    }
}

// Implemented only for the `'static` instantiation; see the note on the
// `string_trait!` impl for why `Trait` requires `Self: 'static`.
impl Trait for HttpHeaderTrait<'static> {
    fn trait_id(&self) -> &ShapeId<'static> {
        &Self::TRAIT_ID
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

string_trait!(
    /// The `@httpQuery` trait — binds a member to a query parameter.
    HttpQueryTrait,
    "smithy.api", "httpQuery"
);

string_trait!(
    /// The `@httpPrefixHeaders` trait — binds a map to prefixed HTTP headers.
    HttpPrefixHeadersTrait,
    "smithy.api", "httpPrefixHeaders"
);

annotation_trait!(
    /// The `@httpLabel` trait — binds a member to a URI label.
    HttpLabelTrait,
    "smithy.api", "httpLabel"
);

annotation_trait!(
    /// The `@httpPayload` trait — binds a member to the HTTP body.
    HttpPayloadTrait,
    "smithy.api", "httpPayload"
);

annotation_trait!(
    /// The `@httpQueryParams` trait — binds a map to query parameters.
    HttpQueryParamsTrait,
    "smithy.api", "httpQueryParams"
);

annotation_trait!(
    /// The `@httpResponseCode` trait — binds a member to the HTTP status code.
    HttpResponseCodeTrait,
    "smithy.api", "httpResponseCode"
);

/// The `@http` trait — defines the HTTP method, URI pattern, and status code for an operation.
///
/// This is an operation-level trait that is included on the input schema for
/// convenience, so that the protocol serializer can construct the correct
/// request without needing a separate operation schema.
///
/// The URI pattern may contain `{label}` placeholders that are substituted
/// at serialization time with percent-encoded values from `@httpLabel` members.
#[derive(Debug, Clone)]
pub struct HttpTrait<'a> {
    method: &'a str,
    uri: &'a str,
    code: u16,
}

impl<'a> HttpTrait<'a> {
    /// Creates a new `HttpTrait`. If `code` is `None`, defaults to `200`.
    ///
    /// `method` and `uri` are borrowed for `'a`. Without a non-`'static` `'a`, a
    /// runtime-materialized schema has no method and no URI, so no request can
    /// be constructed from it at all.
    pub const fn new(method: &'a str, uri: &'a str, code: Option<u16>) -> Self {
        Self {
            method,
            uri,
            code: match code {
                Some(c) => c,
                None => 200,
            },
        }
    }

    /// The HTTP method (e.g., `"GET"`, `"POST"`, `"PUT"`).
    pub fn method(&self) -> &'a str {
        self.method
    }

    /// The URI pattern (e.g., `"/resource/{id}"`).
    ///
    /// May contain `{label}` placeholders that correspond to `@httpLabel` members.
    /// The protocol serializer substitutes these with percent-encoded values
    /// collected during member serialization.
    pub fn uri(&self) -> &'a str {
        self.uri
    }

    /// The HTTP status code for a successful response. Defaults to `200`.
    pub fn code(&self) -> u16 {
        self.code
    }
}

// --- Streaming traits ---

annotation_trait!(
    /// The `@streaming` trait — marks a blob or union as streaming.
    StreamingTrait,
    "smithy.api", "streaming"
);

annotation_trait!(
    /// The `@eventHeader` trait — binds a member to an event stream header.
    EventHeaderTrait,
    "smithy.api", "eventHeader"
);

annotation_trait!(
    /// The `@eventPayload` trait — binds a member to an event stream payload.
    EventPayloadTrait,
    "smithy.api", "eventPayload"
);

// --- Documentation / behavior traits ---

annotation_trait!(
    /// The `@sensitive` trait — marks data as sensitive for logging redaction.
    SensitiveTrait,
    "smithy.api", "sensitive"
);

// --- Endpoint traits ---

annotation_trait!(
    /// The `@hostLabel` trait — binds a member to a host prefix label.
    HostLabelTrait,
    "smithy.api", "hostLabel"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downcast_json_name() {
        let t: Box<dyn Trait> = Box::new(JsonNameTrait::new("userName"));
        assert_eq!(t.trait_id().as_str(), "smithy.api#jsonName");
        // The downcast target is the `'static` instantiation: `Trait` requires
        // `Any`, which is only implemented for `JsonNameTrait<'static>`.
        let json_name = t.as_any().downcast_ref::<JsonNameTrait<'static>>().unwrap();
        assert_eq!(json_name.value(), "userName");
    }

    #[test]
    fn downcast_sensitive() {
        let t: Box<dyn Trait> = Box::new(SensitiveTrait);
        assert_eq!(t.trait_id().as_str(), "smithy.api#sensitive");
        assert!(t.as_any().downcast_ref::<SensitiveTrait>().is_some());
    }

    /// The HTTP binder's allocation-free header insert depends on
    /// `value_static()` being `Some` for every `@httpHeader` a schema can
    /// currently hold. If a `Borrowed` constructor is ever added, this test
    /// keeps passing and the binder falls back to an owned name rather than
    /// silently losing the header.
    #[test]
    fn http_header_value_static_is_some() {
        const H: HttpHeaderTrait<'static> = HttpHeaderTrait::new("x-amz-request-id");
        assert_eq!(H.value(), "x-amz-request-id");
        assert_eq!(H.value_static(), Some("x-amz-request-id"));
    }

    #[test]
    fn timestamp_format_parsing() {
        let t = TimestampFormatTrait::new(TimestampFormat::EpochSeconds);
        assert_eq!(t.format(), TimestampFormat::EpochSeconds);
        assert_eq!(t.trait_id().as_str(), "smithy.api#timestampFormat");
    }
}

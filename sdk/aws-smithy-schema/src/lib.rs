/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/* Automatically managed default lints */
#![cfg_attr(docsrs, feature(doc_cfg))]
/* End of automatically managed default lints */
#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]

//! Runtime schema types for Smithy shapes.
//!
//! This crate provides the core types for representing Smithy schemas at
//! runtime, enabling protocol-agnostic serialization and deserialization.
//! The two central types are [`Schema`] (a runtime descriptor of a Smithy
//! shape) and [`ShapeId`] (a Smithy shape identifier). Both are parameterized
//! over a lifetime `'a` that names the data their string fields and member
//! references borrow from.
//!
//! # API stability
//!
//! <div class="warning">
//!
//! **This crate's version number does not mean its API is settled.** It is
//! versioned `1.x` because the generated SDK crates depend on it and a stable
//! SDK cannot depend on a `0.x` crate. We might need to iterate on it in semver-incompatible
//! ways if we find bugs as schema serde is rolled out. Treat it as a supporting
//! crate for generated code that happens to be publicly reachable, but do not
//! rely on it directly.
//!
//! </div>
//!
//! What that means in practice:
//!
//! * **Prefer the API your generated client exposes:** Protocol selection,
//!   document conversion, and error reification are all reachable from the
//!   generated crate. Reaching directly into this crate couples you to
//!   internals of the serde pipeline that the SDK is free to re-plumb.
//! * **Items marked `#[doc(hidden)]` are not part of the public API at all.**
//!   They exist for generated code and the `shape_id!` macro to call, and may
//!   change or disappear in any release.
//!
//! If you do build on these types directly, pin an exact version and budget
//! time to adapt when you upgrade.
//!
//! # Construction patterns
//!
//! ## `Schema<'static>` — the codegen-emitted form
//!
//! Generated SDK code emits every schema as a `static` of type
//! `Schema<'static>`, built at compile time via `const fn` constructors and
//! `with_*` setters. The Smithy prelude entries in this crate
//! ([`prelude::STRING`], [`prelude::INTEGER`], etc.) follow the same pattern.
//! Because the entire schema graph lives in the binary's data segment, there
//! is no startup cost and no heap allocation on the hot serde path.
//!
//! ```
//! use aws_smithy_schema::{shape_id, Schema, ShapeId, ShapeType};
//!
//! const SHAPE_ID: ShapeId<'static> = shape_id!("ns", "MyShape");
//! const MY_SHAPE_SCHEMA: Schema<'static> = Schema::new(SHAPE_ID, ShapeType::String);
//! assert_eq!(MY_SHAPE_SCHEMA.shape_id().as_str(), "ns#MyShape");
//! ```
//!
//! ## `Schema<'a>` — runtime construction
//!
//! Hand-written code may construct schemas at any lifetime; this is the path
//! used by test fixtures and (in the future) by dynamic clients that load a
//! Smithy model at runtime. Every value referenced by the schema must outlive
//! the schema itself; the borrow checker enforces this. See
//! [`Schema::new_struct`] for a worked example.
//!
//! Wire-format trait values follow the same rule and may borrow from storage
//! the caller owns, such as an arena holding the parsed model text — with one
//! exception. [`Schema::with_http_header`] requires `&'static str`, because the
//! HTTP binder passes header names to an API that needs
//! `Cow<'static, str>`. Use [`intern_header_name`] to obtain a suitable
//! `&'static str` from a runtime string; it deduplicates, so the cost is
//! bounded by the number of distinct header names in the model.
//!
//! # Trait maps and the `LazyLock` discipline
//!
//! The serde traits a schema cares about ([`@jsonName`][traits::JsonNameTrait],
//! HTTP bindings, etc.) are stored as inline typed `Option` fields on the
//! schema and accessed via direct field reads. Unknown or custom traits go
//! through a fallback [`TraitMap`] reachable from [`Schema::with_traits`].
//!
//! [`Schema::with_traits`] accepts an `&'a std::sync::LazyLock<TraitMap>`, so
//! the `LazyLock` must outlive any schema that references it. Codegen places
//! both in statics (the `LazyLock` is `'static`, the schema is
//! `Schema<'static>`); runtime-constructed schemas must arrange the lifetimes
//! manually using standard borrow-check discipline.
//!
//! # Variance
//!
//! `Schema<'a>`, `ShapeId<'a>`, and the typed trait wrappers are covariant in
//! `'a`. Covariance is what
//! lets a `&'static Schema<'static>` (the codegen form) coerce implicitly
//! into a `&Schema<'_>` argument at any call site, with no annotation needed
//! at the call site. Compile-time assertion functions in this crate enforce
//! covariance; if a future field change would break it, the build fails
//! before any downstream code is affected.

mod schema {
    pub(crate) mod payload_hint;
    pub(crate) mod shape_id;
    pub(crate) mod shape_type;
    pub(crate) mod trait_map;
    pub(crate) mod trait_type;
    pub(crate) mod traits;

    pub(crate) mod codec;
    pub(crate) mod document;
    pub(crate) mod error_envelope;
    pub(crate) mod header_omit_settings;
    pub(crate) mod http_protocol;
    pub(crate) mod prelude;
    pub(crate) mod protocol;
    pub(crate) mod registry;
    pub(crate) mod serde;
}

pub use schema::payload_hint::PayloadHint;
pub use schema::shape_id::ShapeId;
pub use schema::shape_type::ShapeType;
pub use schema::trait_map::TraitMap;
pub use schema::trait_type::Trait;
pub use schema::trait_type::{AnnotationTrait, DocumentTrait, StringTrait};

/// Interns a header name so it can be attached to a runtime-materialized schema.
///
/// [`Schema::with_http_header`] is the one trait setter that requires
/// `&'static str` rather than `&'a str`, because the HTTP binder hands header
/// names to `Headers::insert`, which needs a `Cow<'static, str>`. Every other
/// wire-format trait can borrow from storage the caller owns — an arena holding
/// the parsed model text, for instance — and be dropped freely.
///
/// This function bridges that gap: it takes a runtime string and returns a
/// `&'static str` suitable for [`Schema::with_http_header`], keeping the
/// binder's zero-allocation fast path intact
/// (`HttpHeaderTrait::value_static` returns `Some` for interned names).
///
/// # This leaks, deliberately
///
/// The returned `&'static str` is never freed. Interning is deduplicated, so a
/// given name leaks at most once no matter how many times it is interned; the
/// total is bounded by the number of *distinct* header names passed in, not by
/// the number of calls. Re-materializing the same model repeatedly does not
/// grow memory. For reference, the whole of Amazon S3 uses 158 distinct header
/// names, about 4.4 KiB once.
///
/// Prefer this over a hand-rolled `Box::leak`, which leaks on every call and so
/// grows without bound when placed in a loop.
///
/// # When this is the wrong tool
///
/// Because the memory is never reclaimed, the bound is "every distinct header
/// name this process has ever seen". That is fine for a client that loads its
/// models at startup. It is *not* fine for a long-lived process that keeps
/// ingesting new models, where the total grows without limit and cannot be
/// reclaimed. Such callers should not intern; they need `@httpHeader` names
/// borrowed from their own storage, which is an additive relaxation of
/// [`Schema::with_http_header`] rather than something an interner can provide.
///
/// # Examples
///
/// ```
/// use aws_smithy_schema::intern_header_name;
///
/// // A header name that only exists at runtime.
/// let parsed = String::from("x-amz-request-id");
/// let name: &'static str = intern_header_name(&parsed);
/// assert_eq!(name, "x-amz-request-id");
///
/// // Interning the same name again returns the identical pointer, not a copy.
/// assert!(std::ptr::eq(name, intern_header_name("x-amz-request-id")));
/// ```
pub fn intern_header_name(name: &str) -> &'static str {
    static INTERNED: std::sync::LazyLock<
        std::sync::Mutex<std::collections::HashSet<&'static str>>,
    > = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));

    // The guarded section cannot panic, so a poisoned lock is not reachable in
    // practice; recover rather than propagate a panic if it somehow happens.
    let mut table = INTERNED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    if let Some(&existing) = table.get(name) {
        return existing;
    }
    let leaked: &'static str = Box::leak(name.to_owned().into_boxed_str());
    table.insert(leaked);
    leaked
}

/// Schemas for the Smithy prelude shapes (`smithy.api#String`,
/// `smithy.api#Integer`, and so on).
pub mod prelude {
    pub use crate::schema::prelude::*;
}

/// Shape serialization and deserialization traits and their error type.
pub mod serde {
    pub use crate::schema::serde::*;
}

/// Runtime representations of the Smithy serialization traits carried on a schema.
pub mod traits {
    pub use crate::schema::traits::*;
}

/// Format codecs that pair a shape serializer with a matching deserializer.
pub mod codec {
    pub use crate::schema::codec::*;
}

/// `Document` shape serde and the discriminated-document conversion extension.
pub mod document {
    pub use crate::schema::document::*;
}

/// Settings controlling which HTTP headers are omitted during serialization.
pub mod header_omit_settings {
    pub use crate::schema::header_omit_settings::*;
}

/// Client protocol abstraction for serializing requests and deserializing responses.
pub mod protocol {
    pub use crate::schema::protocol::*;
}

/// Shared helpers for parsing AWS protocol error envelopes (error-code
/// sanitization and the awsQueryCompatible header), used by the JSON and CBOR
/// codecs.
pub mod error_envelope {
    pub use crate::schema::error_envelope::*;
}

/// HTTP client-protocol building blocks: HTTP bindings and RPC payloads.
pub mod http_protocol {
    pub use crate::schema::http_protocol::*;
}

/// Runtime type and error registries for resolving shapes by `ShapeId`.
pub mod registry {
    pub use crate::schema::registry::*;
}

use schema::traits as trait_types;

/// A Smithy schema — a lightweight runtime representation of a Smithy shape.
///
/// Contains the shape's ID, type, traits relevant to serialization, and
/// references to member schemas (for aggregate types).
///
/// Schemas are constructed at compile time (via `const`) for generated code
/// and prelude types. The Smithy type system is closed, so no extensibility
/// via trait objects is needed.
#[derive(Debug)]
pub struct Schema<'a> {
    id: ShapeId<'a>,
    shape_type: ShapeType,
    /// Member name if this is a member schema.
    member_name: Option<&'a str>,
    /// Member index for position-based lookup in generated code.
    member_index: Option<usize>,
    /// Shape-type-specific member data.
    members: SchemaMembers<'a>,

    /// The pre-synthesis shape name for synthetic operation input/output
    /// shapes.
    ///
    /// Smithy's `OperationNormalizer` rewrites every operation's input/output
    /// to a synthetic shape named `OperationNameInput` / `OperationNameOutput`;
    /// this field surfaces the name component of
    /// `SyntheticInputTrait::originalId` / `SyntheticOutputTrait::originalId`,
    /// which preserves the user-authored shape name. `None` for non-synthetic
    /// shapes and for member schemas.
    ///
    /// Currently consumed by the REST XML codec to determine the body root
    /// element name when no `@xmlName` overrides it. Distinct from `xml_name`,
    /// which carries an `@xmlName` trait value. Other consumers (logging,
    /// future protocols) may also read this field.
    original_name: Option<&'a str>,

    // -- Known serde trait fields (const-constructable) --
    // IMPORTANT: These fields and their `with_*` setters must stay in sync with
    // `knownTraitSetter` in `SchemaGenerator.kt`. If a new known trait is added
    // here, a corresponding entry must be added in the codegen.
    sensitive: Option<trait_types::SensitiveTrait>,
    json_name: Option<trait_types::JsonNameTrait<'a>>,
    timestamp_format: Option<trait_types::TimestampFormatTrait>,
    xml_name: Option<trait_types::XmlNameTrait<'a>>,
    xml_attribute: Option<trait_types::XmlAttributeTrait>,
    xml_flattened: Option<trait_types::XmlFlattenedTrait>,
    /// Marks an operation output struct whose XML wire format omits the
    /// outer wrapper element (set by codegen for operations carrying the
    /// AWS-customization `S3UnwrappedXmlOutputTrait`). Honored by the XML
    /// codec; ignored by other codecs (so the schema remains protocol-neutral
    /// — runtime protocol swap is unaffected).
    xml_unwrapped_output: bool,
    /// `true` (the default, conservative) means this struct has at least one
    /// member that serializes to the request/response body — i.e., a member
    /// without any HTTP binding trait, OR a member with `@httpPayload`
    /// targeting a struct/union (which provides body framing through the codec).
    ///
    /// `false` (set by codegen via [`with_no_body_members`](Schema::with_no_body_members))
    /// means every member is HTTP-bound (header / query / label / prefix-headers
    /// / query-params, or scalar `@httpPayload` whose bytes go into the request
    /// body raw). The HTTP binding protocol uses this to skip body codec
    /// invocation entirely on the request side: no XML/JSON wrapper element
    /// is opened, no `serialize_members` re-entry through the codec proxy
    /// happens, and the body bytes are never collected (they'd be discarded
    /// anyway). Saves ~15-20% on header-heavy SER cases like S3 PutObject.
    has_body_members: bool,
    /// Records whether an `@httpPayload` member supplies its own body framing,
    /// so the HTTP binding protocol does not have to scan members to find out.
    ///
    /// Defaults to [`PayloadHint::Unknown`], which means "not recorded", the
    /// runtime then derives the answer itself. See [`PayloadHint`] for why that
    /// fallback is permanent rather than transitional.
    payload_hint: PayloadHint,
    xml_namespace: Option<trait_types::XmlNamespaceTrait<'a>>,
    /// Deliberately pinned to `'static` while the other seven trait values
    /// carry `'a`. `Headers::insert` takes `impl AsHeaderComponent`, whose
    /// zero-allocation impl is for `&'static str`
    /// (`aws-smithy-runtime-api`'s `MaybeStatic = Cow<'static, str>`), and the
    /// HTTP binder receives member schemas through `ShapeSerializer` methods
    /// whose `&Schema<'_>` lifetime is unrelated to the binder's.
    ///
    /// The field carries `'a` like every other trait, and the `'static`
    /// requirement lives on the *constructor* instead:
    /// [`with_http_header`](Schema::with_http_header) takes `&'static str`, and
    /// `HttpHeaderTrait::value_static` recovers it for the binder. Keeping the
    /// restriction at the constructor rather than in the type means relaxing it
    /// later is additive rather than a breaking signature change, and it avoids
    /// pinning `ClientProtocol::serialize_request` — which would force every
    /// protocol to take a `'static` schema.
    ///
    /// So: runtime-materialized schemas currently set header names from
    /// interned/leaked strings, while every other wire-format trait already
    /// accepts an arbitrary lifetime.
    http_header: Option<trait_types::HttpHeaderTrait<'a>>,
    http_label: Option<trait_types::HttpLabelTrait>,
    http_payload: Option<trait_types::HttpPayloadTrait>,
    http_prefix_headers: Option<trait_types::HttpPrefixHeadersTrait<'a>>,
    http_query: Option<trait_types::HttpQueryTrait<'a>>,
    http_query_params: Option<trait_types::HttpQueryParamsTrait>,
    http_response_code: Option<trait_types::HttpResponseCodeTrait>,
    /// The `@http` trait — an operation-level trait included on the input schema
    /// for convenience so the protocol serializer can construct the request URI.
    http: Option<trait_types::HttpTrait<'a>>,
    streaming: Option<trait_types::StreamingTrait>,
    event_header: Option<trait_types::EventHeaderTrait>,
    event_payload: Option<trait_types::EventPayloadTrait>,
    host_label: Option<trait_types::HostLabelTrait>,
    media_type: Option<trait_types::MediaTypeTrait<'a>>,

    /// Fallback for unknown/custom traits. `None` in const contexts (no allocation).
    traits: Option<&'a std::sync::LazyLock<TraitMap>>,
}

/// Shape-type-specific member references.
#[derive(Debug)]
enum SchemaMembers<'a> {
    /// No members (simple types).
    None,
    /// Structure or union members.
    Struct { members: &'a [&'a Schema<'a>] },
    /// List member schema.
    List { member: &'a Schema<'a> },
    /// Map key and value schemas.
    Map {
        key: &'a Schema<'a>,
        value: &'a Schema<'a>,
    },
}

// ---------- Variance assertions ----------
//
// `Schema<'a>`, `ShapeId<'a>`, and the eight trait-value wrappers must remain
// **covariant** in `'a`. Covariance
// is what lets `&'static Schema<'static>` (the codegen-emitted form) coerce
// implicitly into `&Schema<'_>` at call sites — without it, every call would
// need an explicit lifetime annotation.
//
// Today all of them are covariant by construction: every field is of the form
// `&'a T` or contains another covariant `'a`-parameterized type. There is no
// `&'a mut T`, no `fn(&'a T)`, and no interior mutability tying `'a` invariantly.
//
// These assertion functions make the covariance requirement *load-bearing on
// the build*. If a future field changes that property — e.g. someone adds a
// `RefCell` holding a `&'a` reference, an `fn(&'a Self)` field, or a `*mut T`
// with `'a` — the function bodies will fail to type-check and the build breaks
// before any downstream code is affected.
//
// Compile-time only; zero runtime cost.

#[allow(dead_code)]
fn _assert_schema_covariant<'a, 'b: 'a>(s: &'b Schema<'b>) -> &'a Schema<'a> {
    s
}

#[allow(dead_code)]
fn _assert_shape_id_covariant<'a, 'b: 'a>(id: ShapeId<'b>) -> ShapeId<'a> {
    id
}

// The eight trait-value wrappers carry `'a` too, and the same reasoning
// applies: a codegen-emitted `JsonNameTrait<'static>` has to be usable
// wherever a `JsonNameTrait<'_>` is expected. `HttpHeaderTrait` is included
// even though its only public constructor takes `&'static str` — its private
// `HeaderName<'a>` enum has a `Borrowed(&'a str)` arm, and this assertion is
// what keeps that arm covariant if it ever becomes constructible.
macro_rules! assert_wrapper_covariant {
    ($fn_name:ident, $wrapper:ident) => {
        #[allow(dead_code)]
        fn $fn_name<'a, 'b: 'a>(t: trait_types::$wrapper<'b>) -> trait_types::$wrapper<'a> {
            t
        }
    };
}

assert_wrapper_covariant!(_assert_json_name_covariant, JsonNameTrait);
assert_wrapper_covariant!(_assert_xml_name_covariant, XmlNameTrait);
assert_wrapper_covariant!(_assert_media_type_covariant, MediaTypeTrait);
assert_wrapper_covariant!(_assert_http_query_covariant, HttpQueryTrait);
assert_wrapper_covariant!(
    _assert_http_prefix_headers_covariant,
    HttpPrefixHeadersTrait
);
assert_wrapper_covariant!(_assert_http_header_covariant, HttpHeaderTrait);
assert_wrapper_covariant!(_assert_xml_namespace_covariant, XmlNamespaceTrait);
assert_wrapper_covariant!(_assert_http_covariant, HttpTrait);

impl<'a> Schema<'a> {
    /// Default values for all trait fields (should only be used by constructors as a spread source).
    ///
    /// Implemented as a `const fn` rather than a `const` so it can be
    /// parameterized over the schema lifetime — `const` items cannot
    /// have lifetime parameters in stable Rust.
    const fn empty_traits() -> Schema<'a> {
        Schema {
            id: ShapeId::<'a>::from_parts("", "", ""),
            shape_type: ShapeType::Boolean,
            member_name: None,
            member_index: None,
            members: SchemaMembers::None,
            original_name: None,
            sensitive: None,
            json_name: None,
            timestamp_format: None,
            xml_name: None,
            xml_attribute: None,
            xml_flattened: None,
            xml_unwrapped_output: false,
            has_body_members: true,
            payload_hint: PayloadHint::Unknown,
            xml_namespace: None,
            http_header: None,
            http_label: None,
            http_payload: None,
            http_prefix_headers: None,
            http_query: None,
            http_query_params: None,
            http_response_code: None,
            http: None,
            streaming: None,
            event_header: None,
            event_payload: None,
            host_label: None,
            media_type: None,
            traits: None,
        }
    }

    /// Creates a schema for a simple type (no members).
    pub const fn new(id: ShapeId<'a>, shape_type: ShapeType) -> Self {
        Self {
            id,
            shape_type,
            ..Self::empty_traits()
        }
    }

    /// Creates a schema for a structure or union type.
    ///
    /// `id` and `members` are borrowed for `'a`, so both must outlive the
    /// returned schema. For codegen-emitted schemas this is always `'static`
    /// to `'static`. Hand-written runtime construction can use any lifetime
    /// — typically the surrounding function body's:
    ///
    /// ```
    /// use aws_smithy_schema::{Schema, ShapeId, ShapeType};
    ///
    /// let id = ShapeId::from_parts("ns#Foo", "ns", "Foo");
    /// let member_x = Schema::new_member(
    ///     ShapeId::from_parts("smithy.api#String", "smithy.api", "String"),
    ///     ShapeType::String,
    ///     "x",
    ///     0,
    /// );
    /// let members: [&Schema<'_>; 1] = [&member_x];
    /// let schema = Schema::new_struct(id, ShapeType::Structure, &members);
    ///
    /// assert_eq!(schema.shape_id().as_str(), "ns#Foo");
    /// assert_eq!(schema.shape_type(), ShapeType::Structure);
    /// ```
    ///
    /// `id`, `member_x`, `members`, and `schema` all share the surrounding
    /// scope's lifetime; the borrow checker enforces that `members[0]`
    /// outlives `schema`.
    pub const fn new_struct(
        id: ShapeId<'a>,
        shape_type: ShapeType,
        members: &'a [&'a Schema<'a>],
    ) -> Self {
        Self {
            id,
            shape_type,
            members: SchemaMembers::Struct { members },
            ..Self::empty_traits()
        }
    }

    /// Creates a schema for a list type.
    pub const fn new_list(id: ShapeId<'a>, member: &'a Schema<'a>) -> Self {
        Self {
            id,
            shape_type: ShapeType::List,
            members: SchemaMembers::List { member },
            ..Self::empty_traits()
        }
    }

    /// Creates a schema for a map type.
    pub const fn new_map(id: ShapeId<'a>, key: &'a Schema<'a>, value: &'a Schema<'a>) -> Self {
        Self {
            id,
            shape_type: ShapeType::Map,
            members: SchemaMembers::Map { key, value },
            ..Self::empty_traits()
        }
    }

    /// Creates a member schema wrapping a target schema.
    pub const fn new_member(
        id: ShapeId<'a>,
        shape_type: ShapeType,
        member_name: &'a str,
        member_index: usize,
    ) -> Self {
        Self {
            id,
            shape_type,
            member_name: Some(member_name),
            member_index: Some(member_index),
            ..Self::empty_traits()
        }
    }

    /// Returns the Shape ID of this schema.
    ///
    /// The returned reference's outer lifetime is the receiver's; the
    /// inner `ShapeId<'a>` carries the data's lifetime, which is
    /// `'static` for codegen-emitted schemas (preserving the
    /// `&'static str` accessor guarantees that downstream
    /// optimizations rely on).
    pub fn shape_id(&self) -> &ShapeId<'a> {
        &self.id
    }

    /// Returns the shape type.
    pub fn shape_type(&self) -> ShapeType {
        self.shape_type
    }

    /// Returns the fallback trait map for unknown/custom traits.
    pub fn traits(&self) -> Option<&TraitMap> {
        self.traits.map(|lazy| &**lazy)
    }

    // -- Known trait accessors --

    /// Returns the `@sensitive` trait if present.
    pub fn sensitive(&self) -> Option<&trait_types::SensitiveTrait> {
        self.sensitive.as_ref()
    }

    /// Returns the `@jsonName` value if present.
    pub fn json_name(&self) -> Option<&trait_types::JsonNameTrait<'a>> {
        self.json_name.as_ref()
    }

    /// Returns the `@timestampFormat` if present.
    pub fn timestamp_format(&self) -> Option<&trait_types::TimestampFormatTrait> {
        self.timestamp_format.as_ref()
    }

    /// Returns the `@xmlName` value if present.
    pub fn xml_name(&self) -> Option<&trait_types::XmlNameTrait<'a>> {
        self.xml_name.as_ref()
    }

    /// Returns the `@xmlNamespace` value if present.
    pub fn xml_namespace(&self) -> Option<&trait_types::XmlNamespaceTrait<'a>> {
        self.xml_namespace.as_ref()
    }

    /// Returns `true` if this member has the `@xmlAttribute` trait.
    pub fn xml_attribute(&self) -> bool {
        self.xml_attribute.is_some()
    }

    /// Returns `true` if this member has the `@xmlFlattened` trait.
    pub fn xml_flattened(&self) -> bool {
        self.xml_flattened.is_some()
    }

    /// Returns `true` if this struct's XML wire format omits the outer
    /// wrapper element. See field doc for details.
    pub fn xml_unwrapped_output(&self) -> bool {
        self.xml_unwrapped_output
    }

    /// Returns `true` if this struct has at least one member that serializes
    /// to the request/response body, `false` if every member is HTTP-bound.
    pub fn has_body_members(&self) -> bool {
        self.has_body_members
    }

    /// Returns the recorded [`PayloadHint`], or [`PayloadHint::Unknown`] if none
    /// was recorded.
    ///
    /// Callers MUST handle `Unknown` by deriving the answer themselves; it is
    /// the default for every hand-constructed schema and for schemas generated
    /// before codegen began recording it.
    pub fn payload_hint(&self) -> PayloadHint {
        self.payload_hint
    }

    /// Returns `true` if this member schema has any HTTP response binding trait
    /// (`@httpHeader`, `@httpResponseCode`, `@httpPrefixHeaders`, or `@httpPayload`).
    pub fn has_http_response_binding(&self) -> bool {
        self.http_header.is_some()
            || self.http_response_code.is_some()
            || self.http_prefix_headers.is_some()
            || self.http_payload.is_some()
    }

    /// Returns the `@httpHeader` value if present.
    ///
    /// Use [`HttpHeaderTrait::value_static`](trait_types::HttpHeaderTrait::value_static)
    /// rather than `value()` when inserting into `Headers`, which needs a
    /// `'static` name; see the field's comment for why.
    pub fn http_header(&self) -> Option<&trait_types::HttpHeaderTrait<'a>> {
        self.http_header.as_ref()
    }

    /// Returns the `@httpQuery` value if present.
    pub fn http_query(&self) -> Option<&trait_types::HttpQueryTrait<'a>> {
        self.http_query.as_ref()
    }

    /// Returns the `@httpLabel` trait if present.
    pub fn http_label(&self) -> Option<&trait_types::HttpLabelTrait> {
        self.http_label.as_ref()
    }

    /// Returns the `@httpPayload` trait if present.
    pub fn http_payload(&self) -> Option<&trait_types::HttpPayloadTrait> {
        self.http_payload.as_ref()
    }

    /// Returns the `@httpPrefixHeaders` value if present.
    pub fn http_prefix_headers(&self) -> Option<&trait_types::HttpPrefixHeadersTrait<'a>> {
        self.http_prefix_headers.as_ref()
    }

    /// Returns the `@mediaType` trait if present.
    pub fn media_type(&self) -> Option<&trait_types::MediaTypeTrait<'a>> {
        self.media_type.as_ref()
    }

    /// Returns the `@httpQueryParams` trait if present.
    pub fn http_query_params(&self) -> Option<&trait_types::HttpQueryParamsTrait> {
        self.http_query_params.as_ref()
    }

    /// Returns the `@httpResponseCode` trait if present.
    pub fn http_response_code(&self) -> Option<&trait_types::HttpResponseCodeTrait> {
        self.http_response_code.as_ref()
    }

    /// Returns the `@http` trait if present.
    ///
    /// This is an operation-level trait included on the input schema for
    /// convenience so the protocol serializer can construct the request URI.
    pub fn http(&self) -> Option<&trait_types::HttpTrait<'a>> {
        self.http.as_ref()
    }

    // -- Const setters for builder-style construction in generated code --

    /// Sets the original (pre-synthesis) shape name for synthetic operation
    /// input/output shapes. See [`Schema::original_name`] for semantics.
    pub const fn with_original_name(mut self, name: &'a str) -> Self {
        self.original_name = Some(name);
        self
    }

    /// Attaches key and value member schemas to a map member schema.
    /// Used by the XML codec to resolve `<key>` and `<value>` element names.
    pub const fn with_map_members(mut self, key: &'a Schema<'a>, value: &'a Schema<'a>) -> Self {
        self.members = SchemaMembers::Map { key, value };
        self
    }

    /// Sets the list member schema on a member schema that targets a list.
    pub const fn with_list_member(mut self, member: &'a Schema<'a>) -> Self {
        self.members = SchemaMembers::List { member };
        self
    }

    /// Sets the `@sensitive` trait.
    pub const fn with_sensitive(mut self) -> Self {
        self.sensitive = Some(trait_types::SensitiveTrait);
        self
    }

    // -- Trait-wrapper setters --
    //
    // The setters below construct typed trait wrappers (`JsonNameTrait`,
    // `XmlNameTrait`, `HttpTrait`, etc.). All of them take `&'a str` — the
    // schema's data lifetime — so a schema materialized at runtime can carry
    // wire-format traits borrowed from storage the caller owns, such as an
    // arena holding the parsed model text. Codegen-emitted schemas pass string
    // literals, so for them `'a` is `'static` and nothing changes.
    //
    // `with_http_header` is the one exception and takes `&'static str`; see its
    // own doc comment and the `http_header` field comment for why, and note the
    // restriction is at the constructor rather than in the type, so relaxing it
    // later is additive.
    //
    // INVARIANT — do not introduce a trait field that needs drop.
    //
    // Every `with_*` setter is a `const fn`, and assigning to a field drops the
    // field's previous value. `Schema::new_*` likewise drops the remainder of
    // the base value via `..Self::empty_traits()`. A `const fn` body cannot run
    // destructors, so a single field with drop glue makes *every* setter and
    // *every* constructor illegal in const context (E0493). That matters
    // because generated schemas are const-initialized statics:
    //
    //     static FOO_SCHEMA: Schema<'static> =
    //         Schema::new_member(..).with_http_header("x-amz-...");
    //
    // and a const initializer cannot allocate. So trait values are `&'a str`
    // and must stay borrow-only: `Cow<'static, str>`, `String`, or any enum
    // with an owning arm are all ruled out, however convenient they look. A
    // borrowed-only enum is fine — `HttpHeaderTrait` uses one — because
    // references have no drop glue.
    //
    // Keeping the fields borrow-only also preserves the niche optimization
    // that makes `Option<JsonNameTrait<'a>>` 16 bytes rather than 24, which at
    // SDK scale is thousands of schemas per crate.

    /// Sets the `@jsonName` trait.
    pub const fn with_json_name(mut self, value: &'a str) -> Self {
        self.json_name = Some(trait_types::JsonNameTrait::new(value));
        self
    }

    /// Sets the `@timestampFormat` trait.
    pub const fn with_timestamp_format(mut self, format: trait_types::TimestampFormat) -> Self {
        self.timestamp_format = Some(trait_types::TimestampFormatTrait::new(format));
        self
    }

    /// Sets the `@xmlName` trait.
    pub const fn with_xml_name(mut self, value: &'a str) -> Self {
        self.xml_name = Some(trait_types::XmlNameTrait::new(value));
        self
    }

    /// Sets the `@xmlAttribute` trait.
    pub const fn with_xml_attribute(mut self) -> Self {
        self.xml_attribute = Some(trait_types::XmlAttributeTrait);
        self
    }

    /// Sets the `@xmlFlattened` trait.
    pub const fn with_xml_flattened(mut self) -> Self {
        self.xml_flattened = Some(trait_types::XmlFlattenedTrait);
        self
    }

    /// Marks the struct as an unwrapped XML output. See field doc for details.
    pub const fn with_xml_unwrapped_output(mut self) -> Self {
        self.xml_unwrapped_output = true;
        self
    }

    /// Marks this struct as having no body members — every member is HTTP-bound.
    /// See [`has_body_members`](Schema::has_body_members) for what this enables.
    pub const fn with_no_body_members(mut self) -> Self {
        self.has_body_members = false;
        self
    }

    /// Records whether an `@httpPayload` member supplies its own body framing.
    ///
    /// Setting this lets the HTTP binding protocol skip a per-request scan of
    /// this schema's members. It is an optimization only: leaving it at
    /// [`PayloadHint::Unknown`] (the default) produces identical output. See
    /// [`PayloadHint`] for why.
    pub const fn with_payload_hint(mut self, hint: PayloadHint) -> Self {
        self.payload_hint = hint;
        self
    }

    /// Sets the `@httpHeader` trait.
    ///
    /// Takes `&'static str`, unlike the other wire-format setters, so the
    /// binder's `Headers::insert` fast path stays allocation-free. A schema
    /// built at runtime must supply an interned or leaked header name.
    ///
    /// Relaxing this is additive: a `with_http_header_borrowed(&'a str)`
    /// companion can be added without changing this signature or the getter's.
    pub const fn with_http_header(mut self, value: &'static str) -> Self {
        self.http_header = Some(trait_types::HttpHeaderTrait::new(value));
        self
    }

    /// Sets the `@httpLabel` trait.
    pub const fn with_http_label(mut self) -> Self {
        self.http_label = Some(trait_types::HttpLabelTrait);
        self
    }

    /// Sets the `@httpPayload` trait.
    pub const fn with_http_payload(mut self) -> Self {
        self.http_payload = Some(trait_types::HttpPayloadTrait);
        self
    }

    /// Sets the `@httpPrefixHeaders` trait.
    pub const fn with_http_prefix_headers(mut self, value: &'a str) -> Self {
        self.http_prefix_headers = Some(trait_types::HttpPrefixHeadersTrait::new(value));
        self
    }

    /// Sets the `@httpQuery` trait.
    pub const fn with_http_query(mut self, value: &'a str) -> Self {
        self.http_query = Some(trait_types::HttpQueryTrait::new(value));
        self
    }

    /// Sets the `@httpQueryParams` trait.
    pub const fn with_http_query_params(mut self) -> Self {
        self.http_query_params = Some(trait_types::HttpQueryParamsTrait);
        self
    }

    /// Sets the `@httpResponseCode` trait.
    pub const fn with_http_response_code(mut self) -> Self {
        self.http_response_code = Some(trait_types::HttpResponseCodeTrait);
        self
    }

    /// Sets the `@http` trait (operation-level, included on input schema for convenience).
    pub const fn with_http(mut self, http: trait_types::HttpTrait<'a>) -> Self {
        self.http = Some(http);
        self
    }

    /// Sets the `@streaming` trait.
    pub const fn with_streaming(mut self) -> Self {
        self.streaming = Some(trait_types::StreamingTrait);
        self
    }

    /// Sets the `@eventHeader` trait.
    pub const fn with_event_header(mut self) -> Self {
        self.event_header = Some(trait_types::EventHeaderTrait);
        self
    }

    /// Sets the `@eventPayload` trait.
    pub const fn with_event_payload(mut self) -> Self {
        self.event_payload = Some(trait_types::EventPayloadTrait);
        self
    }

    /// Sets the `@hostLabel` trait.
    pub const fn with_host_label(mut self) -> Self {
        self.host_label = Some(trait_types::HostLabelTrait);
        self
    }

    /// Sets the `@mediaType` trait.
    pub const fn with_media_type(mut self, value: &'a str) -> Self {
        self.media_type = Some(trait_types::MediaTypeTrait::new(value));
        self
    }

    /// Sets the `@xmlNamespace` trait.
    ///
    /// `uri` is the namespace URI; `prefix` optionally declares the
    /// `xmlns:prefix` form. Pass `None` for the default (unprefixed)
    /// `xmlns="uri"` declaration.
    pub const fn with_xml_namespace(mut self, uri: &'a str, prefix: Option<&'a str>) -> Self {
        self.xml_namespace = Some(trait_types::XmlNamespaceTrait::new(uri, prefix));
        self
    }

    /// Sets the fallback trait map for unknown/custom traits.
    ///
    /// The schema must outlive the `LazyLock` it references. For codegen-emitted
    /// schemas this is always `'static` to `'static`. Runtime-built schemas
    /// must arrange the lifetimes via standard borrow-check discipline.
    pub const fn with_traits(mut self, traits: &'a std::sync::LazyLock<TraitMap>) -> Self {
        self.traits = Some(traits);
        self
    }

    /// Returns the member name if this is a member schema.
    ///
    /// Returns `Option<&'a str>` (the schema's data lifetime, not the
    /// receiver's). For `Schema<'static>` (codegen-emitted) this is
    /// `Option<&'static str>` and callers can store the name in
    /// `Cow::Borrowed` or other `'static`-lifetime contexts without
    /// allocating, matching the underlying field type.
    pub fn member_name(&self) -> Option<&'a str> {
        self.member_name
    }

    /// Returns the member index for member schemas.
    ///
    /// This is used internally by generated code for efficient member lookup.
    /// Consumer code should not rely on specific position values as they may change.
    pub fn member_index(&self) -> Option<usize> {
        self.member_index
    }

    /// Returns the original (pre-synthesis) shape name for synthetic operation
    /// input/output shapes.
    ///
    /// `None` for non-synthetic shapes. See the field documentation for full
    /// semantics.
    pub fn original_name(&self) -> Option<&str> {
        self.original_name
    }

    /// Returns the member schema by name (for structures and unions).
    pub fn member_schema(&self, name: &str) -> Option<&Schema<'_>> {
        match &self.members {
            SchemaMembers::Struct { members } => members
                .iter()
                .find(|m| m.member_name == Some(name))
                .copied(),
            _ => None,
        }
    }

    /// Returns the member name and schema by position index (for structures and unions).
    ///
    /// This is an optimization for generated code to avoid string lookups.
    /// Consumer code should not rely on specific position values as they may change.
    pub fn member_schema_by_index(&self, index: usize) -> Option<&Schema<'_>> {
        match &self.members {
            SchemaMembers::Struct { members } => members.get(index).copied(),
            _ => None,
        }
    }

    /// Returns the member schemas (for structures and unions).
    pub fn members(&self) -> &[&Schema<'_>] {
        match &self.members {
            SchemaMembers::Struct { members } => members,
            _ => &[],
        }
    }

    /// Returns the member schema for collections (list member or map value).
    pub fn member(&self) -> Option<&Schema<'_>> {
        match &self.members {
            SchemaMembers::List { member } => Some(member),
            SchemaMembers::Map { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Like [`member`](Self::member) but returns the `'a` (data-lifetime)
    /// borrow that codegen actually stores. Use this when the caller must
    /// hold a reference to a value/element member schema across nested
    /// callbacks without inheriting the parent `&self` borrow's lifetime.
    ///
    /// For `Schema<'static>` (codegen-emitted) this returns
    /// `Option<&'static Schema<'static>>`.
    pub fn member_borrowed(&self) -> Option<&'a Schema<'a>> {
        match &self.members {
            SchemaMembers::List { member } => Some(*member),
            SchemaMembers::Map { value, .. } => Some(*value),
            _ => None,
        }
    }

    /// Returns the key schema for maps.
    pub fn key(&self) -> Option<&Schema<'_>> {
        match &self.members {
            SchemaMembers::Map { key, .. } => Some(key),
            _ => None,
        }
    }

    /// Like [`key`](Self::key) but returns the `'a` (data-lifetime) borrow
    /// that codegen stores. See [`member_borrowed`](Self::member_borrowed).
    pub fn key_borrowed(&self) -> Option<&'a Schema<'a>> {
        match &self.members {
            SchemaMembers::Map { key, .. } => Some(*key),
            _ => None,
        }
    }

    // -- convenience predicates --

    /// Returns true if this is a member schema.
    pub fn is_member(&self) -> bool {
        self.shape_type.is_member()
    }

    /// Returns true if this is a structure schema.
    pub fn is_structure(&self) -> bool {
        self.shape_type == ShapeType::Structure
    }

    /// Returns true if this is a union schema.
    pub fn is_union(&self) -> bool {
        self.shape_type == ShapeType::Union
    }

    /// Returns true if this is a list schema.
    pub fn is_list(&self) -> bool {
        self.shape_type == ShapeType::List
    }

    /// Returns true if this is a map schema.
    pub fn is_map(&self) -> bool {
        self.shape_type == ShapeType::Map
    }

    /// Returns true if this is a blob schema.
    pub fn is_blob(&self) -> bool {
        self.shape_type == ShapeType::Blob
    }

    /// Returns true if this is a string schema.
    pub fn is_string(&self) -> bool {
        self.shape_type == ShapeType::String
    }
}

#[cfg(test)]
mod test {
    use crate::{shape_id, Schema, ShapeId, ShapeType, Trait, TraitMap};

    // Simple test trait implementation
    #[derive(Debug)]
    struct TestTrait {
        id: crate::ShapeId<'static>,
        #[allow(dead_code)]
        value: String,
    }

    impl Trait for TestTrait {
        fn trait_id(&self) -> &crate::ShapeId<'static> {
            &self.id
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_shape_type_simple() {
        assert!(ShapeType::String.is_simple());
        assert!(ShapeType::Integer.is_simple());
        assert!(ShapeType::Boolean.is_simple());
        assert!(!ShapeType::Structure.is_simple());
        assert!(!ShapeType::List.is_simple());
    }

    #[test]
    fn test_shape_type_aggregate() {
        assert!(ShapeType::Structure.is_aggregate());
        assert!(ShapeType::Union.is_aggregate());
        assert!(ShapeType::List.is_aggregate());
        assert!(ShapeType::Map.is_aggregate());
        assert!(!ShapeType::String.is_aggregate());
    }

    #[test]
    fn test_shape_type_member() {
        assert!(ShapeType::Member.is_member());
        assert!(!ShapeType::String.is_member());
        assert!(!ShapeType::Structure.is_member());
    }

    #[test]
    fn test_shape_id_parsing() {
        let id = shape_id!("smithy.api", "String");
        assert_eq!(id.namespace(), "smithy.api");
        assert_eq!(id.shape_name(), "String");
        assert_eq!(id.member_name(), None);
    }

    #[test]
    fn test_shape_id_with_member() {
        let id = shape_id!("com.example", "MyStruct", "member");
        assert_eq!(id.namespace(), "com.example");
        assert_eq!(id.shape_name(), "MyStruct");
        assert_eq!(id.member_name(), Some("member"));
    }

    #[test]
    fn test_trait_map() {
        let mut map = TraitMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);

        let trait_id = shape_id!("smithy.api", "required");
        let test_trait = Box::new(TestTrait {
            id: trait_id.clone(),
            value: "test".to_string(),
        });

        map.insert(test_trait);
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
        assert!(map.contains(&trait_id));

        let retrieved = map.get(&trait_id);
        assert!(retrieved.is_some());
    }

    /// `TraitMap::get` and `contains` accept any `&ShapeId<'_>`.
    /// `get_fqn` / `contains_fqn` accept `&str`. All resolve against the
    /// `'static`-keyed table by fully qualified name.
    #[test]
    fn test_trait_map_cross_lifetime_lookup() {
        let mut map = TraitMap::new();
        let trait_id = shape_id!("smithy.api", "required");
        map.insert(Box::new(TestTrait {
            id: trait_id,
            value: "test".to_string(),
        }));

        // `&str` lookup.
        assert!(map.contains_fqn("smithy.api#required"));
        assert!(map.get_fqn("smithy.api#required").is_some());
        assert!(!map.contains_fqn("smithy.api#missing"));

        // Runtime-built ShapeId lookup.
        let owned_fqn = String::from("smithy.api#required");
        let owned_ns = String::from("smithy.api");
        let owned_name = String::from("required");
        let runtime_id: ShapeId<'_> = ShapeId::from_parts(&owned_fqn, &owned_ns, &owned_name);
        assert!(map.contains(&runtime_id));
        assert!(map.get(&runtime_id).is_some());
    }

    #[test]
    fn test_schema_predicates() {
        let schema = Schema::new(shape_id!("com.example", "MyStruct"), ShapeType::Structure);

        assert!(schema.is_structure());
        assert!(!schema.is_union());
        assert!(!schema.is_list());
        assert!(!schema.is_member());
    }

    #[test]
    fn test_schema_basic() {
        let schema = Schema::new(shape_id!("smithy.api", "String"), ShapeType::String);

        assert_eq!(schema.shape_id().as_str(), "smithy.api#String");
        assert_eq!(schema.shape_type(), ShapeType::String);
        assert!(schema.traits().is_none());
        assert!(schema.member_name().is_none());
        assert!(schema.member_schema("test").is_none());
        assert!(schema.member_schema_by_index(0).is_none());
    }

    /// The gap this test exists to close: `shape_id.rs` already covers runtime
    /// `ShapeId`s, but nothing covered runtime *trait values*, so a `Schema`
    /// could be given the right shape and then not be given a wire format.
    ///
    /// Every string here is owned by a local `Vec<String>` — an arena the
    /// caller controls. Nothing is `'static` and nothing is leaked, which is
    /// the property that makes runtime-materialized schemas practical.
    #[test]
    fn runtime_trait_values_from_an_arena() {
        let arena: Vec<String> = vec![
            String::from("ns#Foo"),              // 0 fqn
            String::from("ns"),                  // 1 namespace
            String::from("Foo"),                 // 2 shape name
            String::from("fieldName"),           // 3 member name
            String::from("WireName"),            // 4 @jsonName
            String::from("wire-elem"),           // 5 @xmlName
            String::from("application/json"),    // 6 @mediaType
            String::from("qparam"),              // 7 @httpQuery
            String::from("http://ns.example/x"), // 8 @xmlNamespace uri
            String::from("px"),                  // 9 @xmlNamespace prefix
        ];

        let member: Schema<'_> = Schema::new_member(
            ShapeId::from_parts(&arena[0], &arena[1], &arena[2]),
            ShapeType::String,
            &arena[3],
            0,
        )
        .with_json_name(&arena[4])
        .with_xml_name(&arena[5])
        .with_media_type(&arena[6])
        .with_http_query(&arena[7])
        .with_xml_namespace(&arena[8], Some(&arena[9]));

        // Read every wire-format trait back off the runtime schema.
        assert_eq!(member.json_name().unwrap().value(), "WireName");
        assert_eq!(member.xml_name().unwrap().value(), "wire-elem");
        assert_eq!(member.media_type().unwrap().value(), "application/json");
        assert_eq!(member.http_query().unwrap().value(), "qparam");
        let ns = member.xml_namespace().unwrap();
        assert_eq!(ns.uri(), "http://ns.example/x");
        assert_eq!(ns.prefix(), Some("px"));
        assert_eq!(member.member_name(), Some("fieldName"));

        // And it composes into a struct schema whose members are readable.
        let members = [&member];
        let schema = Schema::new_struct(
            ShapeId::from_parts(&arena[0], &arena[1], &arena[2]),
            ShapeType::Structure,
            &members,
        );
        assert_eq!(
            schema
                .member_schema("fieldName")
                .unwrap()
                .json_name()
                .unwrap()
                .value(),
            "WireName"
        );
    }

    /// `@http` carries the method and URI, so without runtime support a
    /// dynamically-built operation has no request line at all.
    #[test]
    fn runtime_http_trait_from_an_arena() {
        let method = String::from("PATCH");
        let uri = String::from("/things/{id}");

        let schema = Schema::new(shape_id!("ns", "Op"), ShapeType::Structure)
            .with_http(crate::traits::HttpTrait::new(&method, &uri, Some(204)));

        let http = schema.http().unwrap();
        assert_eq!(http.method(), "PATCH");
        assert_eq!(http.uri(), "/things/{id}");
        assert_eq!(http.code(), 204);
    }

    /// A trait value read off a `Schema<'static>` is still `&'static str`, so
    /// the relaxation costs codegen-emitted schemas nothing. This is a
    /// compile-time assertion: the annotation is the test.
    #[test]
    fn static_schema_still_yields_static_trait_values() {
        static S: Schema<'static> =
            Schema::new(shape_id!("ns", "Foo"), ShapeType::String).with_json_name("WireName");

        let value: &'static str = S.json_name().unwrap().value();
        assert_eq!(value, "WireName");
    }

    /// Interning is deduplicated: the same name yields the identical pointer,
    /// so a repeated call leaks nothing further. This is the property that
    /// makes `intern_header_name` safe where a bare `Box::leak` is not.
    #[test]
    fn intern_header_name_dedups_by_pointer() {
        let from_runtime = String::from("x-dedup-probe");

        let a = crate::intern_header_name(&from_runtime);
        let b = crate::intern_header_name("x-dedup-probe");
        let c = crate::intern_header_name(&String::from("x-dedup-probe"));

        assert_eq!(a, "x-dedup-probe");
        assert!(std::ptr::eq(a, b), "second intern must reuse the first");
        assert!(std::ptr::eq(a, c), "third intern must reuse the first");

        // Distinct names are distinct allocations with correct contents.
        let other = crate::intern_header_name("x-dedup-other");
        assert!(!std::ptr::eq(a, other));
        assert_eq!(other, "x-dedup-other");
    }

    /// The point of interning: a header name that only exists at runtime can be
    /// attached to a schema, and the binder still sees a `'static` name, so the
    /// zero-allocation insert path is preserved rather than falling back to an
    /// owned copy.
    #[test]
    fn interned_header_name_preserves_the_binder_fast_path() {
        let arena: Vec<String> = vec![String::from("memberName"), String::from("x-runtime-hdr")];

        let member: Schema<'_> =
            Schema::new_member(shape_id!("ns", "Foo"), ShapeType::String, &arena[0], 0)
                .with_http_header(crate::intern_header_name(&arena[1]));

        // `Some` is what the binder needs; `None` would force an allocation.
        assert_eq!(
            member.http_header().unwrap().value_static(),
            Some("x-runtime-hdr")
        );
        assert_eq!(member.http_header().unwrap().value(), "x-runtime-hdr");
    }
}

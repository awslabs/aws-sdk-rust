/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/* Automatically managed default lints */
#![cfg_attr(docsrs, feature(doc_cfg))]
/* End of automatically managed default lints */

//! Runtime schema types for Smithy shapes.
//!
//! This module provides the core types for representing Smithy schemas at runtime,
//! enabling protocol-agnostic serialization and deserialization.

mod schema {
    pub mod shape_id;
    pub mod shape_type;
    pub mod trait_map;
    pub mod trait_type;
    pub mod traits;

    pub mod codec;
    pub mod http_protocol;
    pub mod prelude;
    pub mod protocol;
    pub mod serde;
}

pub use schema::shape_id::ShapeId;
pub use schema::shape_type::ShapeType;
pub use schema::trait_map::TraitMap;
pub use schema::trait_type::Trait;
pub use schema::trait_type::{AnnotationTrait, DocumentTrait, StringTrait};

pub mod prelude {
    pub use crate::schema::prelude::*;
}

pub mod serde {
    pub use crate::schema::serde::*;
}

pub mod traits {
    pub use crate::schema::traits::*;
}

pub mod codec {
    pub use crate::schema::codec::*;
}

pub mod protocol {
    pub use crate::schema::protocol::*;
}

pub mod http_protocol {
    pub use crate::schema::http_protocol::*;
}

/// A Smithy schema — a lightweight runtime representation of a Smithy shape.
///
/// Contains the shape's ID, type, traits relevant to serialization, and
/// references to member schemas (for aggregate types).
///
/// Schemas are constructed at compile time (via `const`) for generated code
/// and prelude types. The Smithy type system is closed, so no extensibility
/// via trait objects is needed.
use schema::traits as trait_types;

#[derive(Debug)]
pub struct Schema {
    id: ShapeId,
    shape_type: ShapeType,
    /// Member name if this is a member schema.
    member_name: Option<&'static str>,
    /// Member index for position-based lookup in generated code.
    member_index: Option<usize>,
    /// Shape-type-specific member data.
    members: SchemaMembers,

    // -- Known serde trait fields (const-constructable) --
    // IMPORTANT: These fields and their `with_*` setters must stay in sync with
    // `knownTraitSetter` in `SchemaGenerator.kt`. If a new known trait is added
    // here, a corresponding entry must be added in the codegen.
    sensitive: Option<trait_types::SensitiveTrait>,
    json_name: Option<trait_types::JsonNameTrait>,
    timestamp_format: Option<trait_types::TimestampFormatTrait>,
    xml_name: Option<trait_types::XmlNameTrait>,
    xml_attribute: Option<trait_types::XmlAttributeTrait>,
    xml_flattened: Option<trait_types::XmlFlattenedTrait>,
    xml_namespace: Option<trait_types::XmlNamespaceTrait>,
    http_header: Option<trait_types::HttpHeaderTrait>,
    http_label: Option<trait_types::HttpLabelTrait>,
    http_payload: Option<trait_types::HttpPayloadTrait>,
    http_prefix_headers: Option<trait_types::HttpPrefixHeadersTrait>,
    http_query: Option<trait_types::HttpQueryTrait>,
    http_query_params: Option<trait_types::HttpQueryParamsTrait>,
    http_response_code: Option<trait_types::HttpResponseCodeTrait>,
    /// The `@http` trait — an operation-level trait included on the input schema
    /// for convenience so the protocol serializer can construct the request URI.
    http: Option<trait_types::HttpTrait>,
    streaming: Option<trait_types::StreamingTrait>,
    event_header: Option<trait_types::EventHeaderTrait>,
    event_payload: Option<trait_types::EventPayloadTrait>,
    host_label: Option<trait_types::HostLabelTrait>,
    media_type: Option<trait_types::MediaTypeTrait>,

    /// Fallback for unknown/custom traits. `None` in const contexts (no allocation).
    traits: Option<&'static std::sync::LazyLock<TraitMap>>,
}

/// Shape-type-specific member references.
#[derive(Debug)]
enum SchemaMembers {
    /// No members (simple types).
    None,
    /// Structure or union members.
    Struct { members: &'static [&'static Schema] },
    /// List member schema.
    List { member: &'static Schema },
    /// Map key and value schemas.
    Map {
        key: &'static Schema,
        value: &'static Schema,
    },
}

impl Schema {
    /// Default values for all trait fields (should only be used by constructors as a spread source).
    const EMPTY_TRAITS: Self = Self {
        id: ShapeId::from_static("", "", ""),
        shape_type: ShapeType::Boolean,
        member_name: None,
        member_index: None,
        members: SchemaMembers::None,
        sensitive: None,
        json_name: None,
        timestamp_format: None,
        xml_name: None,
        xml_attribute: None,
        xml_flattened: None,
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
    };

    /// Creates a schema for a simple type (no members).
    pub const fn new(id: ShapeId, shape_type: ShapeType) -> Self {
        Self {
            id,
            shape_type,
            ..Self::EMPTY_TRAITS
        }
    }

    /// Creates a schema for a structure or union type.
    pub const fn new_struct(
        id: ShapeId,
        shape_type: ShapeType,
        members: &'static [&'static Schema],
    ) -> Self {
        Self {
            id,
            shape_type,
            members: SchemaMembers::Struct { members },
            ..Self::EMPTY_TRAITS
        }
    }

    /// Creates a schema for a list type.
    pub const fn new_list(id: ShapeId, member: &'static Schema) -> Self {
        Self {
            id,
            shape_type: ShapeType::List,
            members: SchemaMembers::List { member },
            ..Self::EMPTY_TRAITS
        }
    }

    /// Creates a schema for a map type.
    pub const fn new_map(id: ShapeId, key: &'static Schema, value: &'static Schema) -> Self {
        Self {
            id,
            shape_type: ShapeType::Map,
            members: SchemaMembers::Map { key, value },
            ..Self::EMPTY_TRAITS
        }
    }

    /// Creates a member schema wrapping a target schema.
    pub const fn new_member(
        id: ShapeId,
        shape_type: ShapeType,
        member_name: &'static str,
        member_index: usize,
    ) -> Self {
        Self {
            id,
            shape_type,
            member_name: Some(member_name),
            member_index: Some(member_index),
            ..Self::EMPTY_TRAITS
        }
    }

    /// Returns the Shape ID of this schema.
    pub fn shape_id(&self) -> &ShapeId {
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
    pub fn json_name(&self) -> Option<&trait_types::JsonNameTrait> {
        self.json_name.as_ref()
    }

    /// Returns the `@timestampFormat` if present.
    pub fn timestamp_format(&self) -> Option<&trait_types::TimestampFormatTrait> {
        self.timestamp_format.as_ref()
    }

    /// Returns the `@xmlName` value if present.
    pub fn xml_name(&self) -> Option<&trait_types::XmlNameTrait> {
        self.xml_name.as_ref()
    }

    /// Returns the `@httpHeader` value if present.
    /// Returns `true` if this member schema has any HTTP response binding trait
    /// (`@httpHeader`, `@httpResponseCode`, `@httpPrefixHeaders`, or `@httpPayload`).
    pub fn has_http_response_binding(&self) -> bool {
        self.http_header.is_some()
            || self.http_response_code.is_some()
            || self.http_prefix_headers.is_some()
            || self.http_payload.is_some()
    }

    pub fn http_header(&self) -> Option<&trait_types::HttpHeaderTrait> {
        self.http_header.as_ref()
    }

    /// Returns the `@httpQuery` value if present.
    pub fn http_query(&self) -> Option<&trait_types::HttpQueryTrait> {
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
    pub fn http_prefix_headers(&self) -> Option<&trait_types::HttpPrefixHeadersTrait> {
        self.http_prefix_headers.as_ref()
    }

    /// Returns the `@mediaType` trait if present.
    pub fn media_type(&self) -> Option<&trait_types::MediaTypeTrait> {
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
    pub fn http(&self) -> Option<&trait_types::HttpTrait> {
        self.http.as_ref()
    }

    // -- Const setters for builder-style construction in generated code --

    /// Sets the `@sensitive` trait.
    pub const fn with_sensitive(mut self) -> Self {
        self.sensitive = Some(trait_types::SensitiveTrait);
        self
    }

    /// Sets the `@jsonName` trait.
    pub const fn with_json_name(mut self, value: &'static str) -> Self {
        self.json_name = Some(trait_types::JsonNameTrait::new(value));
        self
    }

    /// Sets the `@timestampFormat` trait.
    pub const fn with_timestamp_format(mut self, format: trait_types::TimestampFormat) -> Self {
        self.timestamp_format = Some(trait_types::TimestampFormatTrait::new(format));
        self
    }

    /// Sets the `@xmlName` trait.
    pub const fn with_xml_name(mut self, value: &'static str) -> Self {
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

    /// Sets the `@httpHeader` trait.
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
    pub const fn with_http_prefix_headers(mut self, value: &'static str) -> Self {
        self.http_prefix_headers = Some(trait_types::HttpPrefixHeadersTrait::new(value));
        self
    }

    /// Sets the `@httpQuery` trait.
    pub const fn with_http_query(mut self, value: &'static str) -> Self {
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
    pub const fn with_http(mut self, http: trait_types::HttpTrait) -> Self {
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
    pub const fn with_media_type(mut self, value: &'static str) -> Self {
        self.media_type = Some(trait_types::MediaTypeTrait::new(value));
        self
    }

    /// Sets the `@xmlNamespace` trait.
    pub const fn with_xml_namespace(mut self) -> Self {
        self.xml_namespace = Some(trait_types::XmlNamespaceTrait);
        self
    }

    /// Sets the fallback trait map for unknown/custom traits.
    pub const fn with_traits(mut self, traits: &'static std::sync::LazyLock<TraitMap>) -> Self {
        self.traits = Some(traits);
        self
    }

    /// Returns the member name if this is a member schema.
    pub fn member_name(&self) -> Option<&str> {
        self.member_name
    }

    /// Returns the member index for member schemas.
    ///
    /// This is used internally by generated code for efficient member lookup.
    /// Consumer code should not rely on specific position values as they may change.
    pub fn member_index(&self) -> Option<usize> {
        self.member_index
    }

    /// Returns the member schema by name (for structures and unions).
    pub fn member_schema(&self, name: &str) -> Option<&Schema> {
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
    pub fn member_schema_by_index(&self, index: usize) -> Option<&Schema> {
        match &self.members {
            SchemaMembers::Struct { members } => members.get(index).copied(),
            _ => None,
        }
    }

    /// Returns the member schemas (for structures and unions).
    pub fn members(&self) -> &[&Schema] {
        match &self.members {
            SchemaMembers::Struct { members } => members,
            _ => &[],
        }
    }

    /// Returns the member schema for collections (list member or map value).
    pub fn member(&self) -> Option<&Schema> {
        match &self.members {
            SchemaMembers::List { member } => Some(member),
            SchemaMembers::Map { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Returns the key schema for maps.
    pub fn key(&self) -> Option<&Schema> {
        match &self.members {
            SchemaMembers::Map { key, .. } => Some(key),
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
    use crate::{shape_id, Schema, ShapeType, Trait, TraitMap};

    // Simple test trait implementation
    #[derive(Debug)]
    struct TestTrait {
        id: crate::ShapeId,
        #[allow(dead_code)]
        value: String,
    }

    impl Trait for TestTrait {
        fn trait_id(&self) -> &crate::ShapeId {
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
            id: trait_id,
            value: "test".to_string(),
        });

        map.insert(test_trait);
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
        assert!(map.contains(&trait_id));

        let retrieved = map.get(&trait_id);
        assert!(retrieved.is_some());
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
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! End-to-end test for the wire-format `__type` discriminator lift.
//!
//! Exercises the full pipeline:
//!
//! ```text
//! JSON wire bytes
//!     ↓ JsonCodec::create_deserializer
//! JsonDeserializer<'a>
//!     ↓ read_discriminated_document (lifts top-level __type)
//! DiscriminatedDocument  (wraps a fully-owned aws_smithy_types::Document
//!                         and carries the lifted FQN as Option<String>)
//!     ↓ TypeRegistry::deserialize_document
//! TypeErasedBox
//!     ↓ downcast::<ConcreteShape>()
//! Typed shape with field values asserted
//! ```
//!
//! This is the user-visible payoff of:
//!  * the `__type` lift in `aws-smithy-json::codec::deserializer`, and
//!  * the cross-lifetime registry-lookup APIs on `TypeRegistry`.
//!
//! `Client::registry()` and the discriminated-document deserialize path are only
//! generated when DynamoDB's protocol is on `SchemaSerdeAllowlist`, so this test
//! compiles and runs only while that protocol is enabled there. It is currently
//! **disabled** because the allowlist is empty.
//!
//! TODO(schema-serde): Rust cannot query the codegen allowlist, so this gating
//! is manual — remove the `#![cfg(any())]` below to re-enable this test when
//! awsJson1_0 (or DynamoDB specifically) is re-added to the allowlist.

#![cfg(any())]

use aws_sdk_dynamodb::types::Capacity;
use aws_sdk_dynamodb::Client;
use aws_smithy_json::codec::{JsonCodec, JsonCodecSettings};
use aws_smithy_schema::codec::Codec;

#[test]
fn wire_bytes_with_absolute_type_round_trip_through_registry() {
    // Wire bytes shaped like a real-world JSON document of a Capacity
    // shape, prefixed with `__type` carrying an absolute Smithy shape
    // ID. This is the form a service emits when it includes typed
    // documents in a response.
    //
    // Members are keyed by the Smithy member name (the schema
    // generator preserves these as the `member_name` field), not the
    // snake_case Rust field name.
    let bytes = br#"{"__type":"com.amazonaws.dynamodb#Capacity","ReadCapacityUnits":1.5,"WriteCapacityUnits":2.5,"CapacityUnits":3.0}"#;

    let codec = JsonCodec::default();
    let mut deser = codec.create_deserializer(bytes);
    let doc = deser
        .read_discriminated_document()
        .expect("well-formed JSON parses successfully");

    // Step 1: the deserializer lifted the top-level __type into the
    // discriminator slot of the DiscriminatedDocument wrapper. The
    // unified design carries the discriminator as an owned FQN
    // string rather than a structured ShapeId — namespace and
    // shape-name parsing is the caller's responsibility if needed.
    let fqn = doc
        .discriminator()
        .expect("absolute __type lifted into discriminator");
    assert_eq!(fqn, "com.amazonaws.dynamodb#Capacity");

    // Step 2: __type was dropped from the resulting map; only the
    // body members remain. The data lives on the inner Document
    // (DiscriminatedDocument is a thin wrapper around it).
    let map = doc
        .document()
        .as_object()
        .expect("top-level value is an object");
    assert!(
        !map.contains_key("__type"),
        "lift must drop __type from the resulting map",
    );
    assert_eq!(map.len(), 3);

    // Step 3: the registry looks up the discriminator FQN against
    // its `'static`-keyed table, finds Capacity, and dispatches to
    // the generated deserialize fn.
    let typed = Client::registry()
        .deserialize_document(&doc)
        .expect("Capacity is registered and the document is well-formed");

    // Step 4: downcast to recover the concrete type.
    let capacity = typed
        .downcast::<Capacity>()
        .expect("registered DeserializeFn returns the correct concrete type");

    assert_eq!(capacity.read_capacity_units, Some(1.5));
    assert_eq!(capacity.write_capacity_units, Some(2.5));
    assert_eq!(capacity.capacity_units, Some(3.0));
}

#[test]
fn wire_bytes_with_relative_type_lifts_under_service_namespace() {
    // With a `default_namespace` configured on the codec (which the
    // generated DynamoDB client emits at protocol construction —
    // see `with_default_namespace("com.amazonaws.dynamodb")` in the
    // codegen output), a relative `__type` is resolved into a
    // fully-qualified shape ID and lifted into the discriminator
    // slot.
    //
    // This mirrors what awsJson1_0 services typically emit on the
    // wire: the shape name without a namespace prefix.
    let bytes = br#"{"__type":"Capacity","ReadCapacityUnits":1.5,"WriteCapacityUnits":2.5,"CapacityUnits":3.0}"#;

    // Construct a codec mirroring what the codegen emits for the
    // DynamoDB protocol — namespace-aware. End-to-end through
    // `Client`'s registered `SharedClientProtocol` exercises the
    // same code path; constructing the codec directly here keeps
    // the test focused on the lift logic.
    let codec = JsonCodec::new(
        JsonCodecSettings::builder()
            .default_namespace("com.amazonaws.dynamodb")
            .build(),
    );
    let mut deser = codec.create_deserializer(bytes);
    let doc = deser
        .read_discriminated_document()
        .expect("well-formed JSON parses successfully");

    // Step 1: the relative __type was resolved to its FQN form
    // and lifted into the wrapper's discriminator slot.
    let fqn = doc
        .discriminator()
        .expect("relative __type lifted under default_namespace");
    assert_eq!(fqn, "com.amazonaws.dynamodb#Capacity");

    // Step 2: __type was dropped from the result map (same shape as
    // the absolute case).
    let map = doc
        .document()
        .as_object()
        .expect("top-level value is an object");
    assert!(
        !map.contains_key("__type"),
        "lift must drop __type from the resulting map",
    );
    assert_eq!(map.len(), 3);

    // Step 3: the registry lookup uses the synthesized FQN and
    // dispatches to the generated Capacity::deserialize fn.
    let typed = Client::registry()
        .deserialize_document(&doc)
        .expect("Capacity is registered and the document is well-formed");

    let capacity = typed
        .downcast::<Capacity>()
        .expect("registered DeserializeFn returns the correct concrete type");

    assert_eq!(capacity.read_capacity_units, Some(1.5));
    assert_eq!(capacity.write_capacity_units, Some(2.5));
    assert_eq!(capacity.capacity_units, Some(3.0));
}

#[test]
fn wire_bytes_with_relative_type_does_not_lift_when_namespace_unset() {
    // Without a configured `default_namespace` on the codec, a
    // relative `__type` is left in the map as a regular key — the
    // discriminator remains None. Hand-built `JsonCodec` instances
    // (i.e. those not produced via the codegen-emitted protocol
    // chain) take this path by default. See
    // `wire_bytes_with_relative_type_lifts_under_service_namespace`
    // for the codegen-driven success case.
    let bytes = br#"{"__type":"Capacity","ReadCapacityUnits":1.5}"#;

    let codec = JsonCodec::default();
    let mut deser = codec.create_deserializer(bytes);
    let doc = deser
        .read_discriminated_document()
        .expect("well-formed JSON parses successfully");

    assert!(
        doc.discriminator().is_none(),
        "relative __type without default_namespace must not be lifted",
    );
    let map = doc
        .document()
        .as_object()
        .expect("top-level value is an object");
    assert_eq!(
        map.get("__type").and_then(|d| d.as_string()),
        Some("Capacity"),
    );
}

#[test]
fn wire_bytes_with_unknown_absolute_type_errors_at_registry() {
    // `__type` is well-formed and gets lifted, but the discriminator
    // doesn't match any registered shape. The registry returns the
    // matching `SerdeError::UnknownMember`.
    let bytes = br#"{"__type":"com.example#NotARealShape"}"#;

    let codec = JsonCodec::default();
    let mut deser = codec.create_deserializer(bytes);
    let doc = deser
        .read_discriminated_document()
        .expect("well-formed JSON parses successfully");

    let fqn = doc.discriminator().expect("absolute lift succeeded");
    assert_eq!(fqn, "com.example#NotARealShape");

    let result = Client::registry().deserialize_document(&doc);
    assert!(
        result.is_err(),
        "unregistered shape id must cause registry lookup to fail",
    );
}

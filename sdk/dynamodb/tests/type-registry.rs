/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! End-to-end test for `Client::registry()` (Phase 4 of the Document Types &
//! Type Registries SEP).
//!
//! Builds a `Document` with a discriminator pointing to a real DynamoDB shape
//! (`com.amazonaws.dynamodb#Capacity`), looks the entry up in the registry,
//! deserializes via `TypeRegistry::deserialize_document`, and downcasts to the
//! concrete type to assert field values round-tripped correctly.
//!
//! `Client::registry()` and `Client::error_registry()` are only generated when
//! DynamoDB's protocol is on `SchemaSerdeAllowlist`, so this test compiles and
//! runs only while that protocol is enabled there. It is currently **disabled**
//! because the allowlist is empty.
//!
//! TODO(schema-serde): Rust cannot query the codegen allowlist, so this gating
//! is manual — remove the `#![cfg(any())]` below to re-enable this test when
//! awsJson1_0 (or DynamoDB specifically) is re-added to the allowlist.

#![cfg(any())]

use aws_sdk_dynamodb::types::Capacity;
use aws_sdk_dynamodb::Client;
use aws_smithy_schema::shape_id;
use aws_smithy_types::{DiscriminatedDocument, Document, Number};

#[test]
fn registry_deserialize_document_round_trip() {
    // Build a Document representing a Capacity shape:
    //   { "ReadCapacityUnits": 1.5, "WriteCapacityUnits": 2.5, "CapacityUnits": 3.0 }
    // The map keys must match the Smithy member names (the schema generator
    // uses the original Smithy names as `member_name`, not the snake_case Rust
    // field names).
    let mut members = aws_smithy_types::document::DocumentObject::new();
    members.insert(
        "ReadCapacityUnits".to_owned(),
        Document::Number(Number::Float(1.5)),
    );
    members.insert(
        "WriteCapacityUnits".to_owned(),
        Document::Number(Number::Float(2.5)),
    );
    members.insert(
        "CapacityUnits".to_owned(),
        Document::Number(Number::Float(3.0)),
    );
    let doc = DiscriminatedDocument::new(Document::Object(members))
        .with_discriminator("com.amazonaws.dynamodb#Capacity");

    // Look up via the package-level registry and deserialize.
    let registry = Client::registry();
    let typed = registry
        .deserialize_document(&doc)
        .expect("Capacity is registered and the document is well-formed");

    // Downcast to the concrete type.
    let capacity = typed
        .downcast::<Capacity>()
        .expect("the registered DeserializeFn returns a Capacity");

    assert_eq!(capacity.read_capacity_units, Some(1.5));
    assert_eq!(capacity.write_capacity_units, Some(2.5));
    assert_eq!(capacity.capacity_units, Some(3.0));
}

#[test]
fn registry_returns_none_for_unregistered_shape() {
    // Unknown shape id → registry has no entry → deserialize_document errors.
    let doc = DiscriminatedDocument::new(Document::Object(
        aws_smithy_types::document::DocumentObject::new(),
    ))
    .with_discriminator("com.example#NotARealShape");

    let result = Client::registry().deserialize_document(&doc);
    assert!(result.is_err(), "unregistered shape should error");
}

#[test]
fn registry_errors_when_discriminator_missing() {
    // No discriminator → deserialize_document errors.
    let doc = DiscriminatedDocument::new(Document::Object(
        aws_smithy_types::document::DocumentObject::new(),
    ));

    let result = Client::registry().deserialize_document(&doc);
    assert!(
        result.is_err(),
        "documents without a discriminator should error"
    );
}

#[test]
fn registry_includes_errors() {
    // Per the SEP, the primary type registry includes every structure shape —
    // `@error` structures included. The same shape also lives in
    // `Client::error_registry()`; a shape legitimately appears in both.
    let id = shape_id!("com.amazonaws.dynamodb", "ResourceInUseException");
    assert!(
        Client::registry().schema_for(&id).is_some(),
        "error structures must appear in the primary type registry"
    );
}

#[test]
fn registry_excludes_synthetic_input_output() {
    // Operation input/output shapes have synthetic IDs. They're codegen
    // artifacts and intentionally excluded from the primary registry.
    let synthetic = shape_id!("com.amazonaws.dynamodb.synthetic", "GetItemInput");
    assert!(
        Client::registry().schema_for(&synthetic).is_none(),
        "synthetic operation input/output shapes must not appear in the primary type registry"
    );
}

// ---- Phase 6.4-6.5: error_registry coverage ----

#[test]
fn error_registry_contains_modeled_errors() {
    // The service-wide error registry should hold a schema for every
    // @error-trait shape in the service closure.
    let id = shape_id!("com.amazonaws.dynamodb", "ResourceInUseException");
    assert!(
        Client::error_registry().schema_for(&id).is_some(),
        "error_registry must contain ResourceInUseException"
    );
    let id = shape_id!("com.amazonaws.dynamodb", "ConditionalCheckFailedException");
    assert!(
        Client::error_registry().schema_for(&id).is_some(),
        "error_registry must contain ConditionalCheckFailedException"
    );
}

#[test]
fn error_registry_excludes_non_error_shapes() {
    // Non-error structures (data shapes) MUST NOT appear in the error
    // registry. Capacity is a regular data structure.
    let id = shape_id!("com.amazonaws.dynamodb", "Capacity");
    assert!(
        Client::error_registry().schema_for(&id).is_none(),
        "non-error shapes must not appear in the error registry"
    );
}

#[test]
fn error_registry_deserialize_document_round_trip() {
    use aws_sdk_dynamodb::types::error::ConditionalCheckFailedException;

    // Build a Document representing a ConditionalCheckFailedException with
    // the Smithy-modeled `Message` member set. Members come from the schema's
    // member_name (Smithy convention), not the snake_case Rust field name.
    let mut members = aws_smithy_types::document::DocumentObject::new();
    members.insert(
        "message".to_owned(),
        Document::String("the conditional request failed".to_owned()),
    );
    let doc = DiscriminatedDocument::new(Document::Object(members))
        .with_discriminator("com.amazonaws.dynamodb#ConditionalCheckFailedException");

    let typed = Client::error_registry()
        .deserialize_document(&doc)
        .expect("ConditionalCheckFailedException is registered");
    let err = typed
        .downcast::<ConditionalCheckFailedException>()
        .expect("registered DeserializeFn returns the correct concrete type");

    assert_eq!(err.message(), Some("the conditional request failed"));
}

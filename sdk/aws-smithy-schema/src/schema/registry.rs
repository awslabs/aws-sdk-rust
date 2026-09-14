/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Runtime type registry for resolving schemas by `ShapeId`.
//!
//! A [`TypeRegistry`] maps shape IDs to a pair of (a) the corresponding
//! [`Schema`] and (b) a deserialize function that produces a typed value
//! from a [`ShapeDeserializer`].
//!
//! Registries are the runtime piece behind the SEP's "Type Registry" concept.
//! They support two main use cases:
//!
//! 1. **Document → typed shape conversion.** Given a [`Document`](aws_smithy_types::Document) carrying
//!    a discriminator ([`ShapeId`]), the registry resolves the discriminator
//!    to a `RegistryEntry`, walks the document via [`DocumentShapeDeserializer`],
//!    and returns the typed value as a [`TypeErasedBox`] for the caller to
//!    downcast.
//! 2. **Schema lookup by shape ID.** Given a [`ShapeId`] (e.g. one extracted
//!    from an error envelope discriminator), the registry returns the
//!    corresponding [`Schema`] so generated code can dispatch on it.
//!
//! Registries are normally constructed at the package level by code generation
//! and exposed through `Client::registry()` (per the paired SEP). They can
//! also be composed via [`TypeRegistry::compose`] to combine, for example,
//! a service's primary registry with an extension package's registry.
//!
//! See the Smithy SEP "Document Types and Type Registries" for the
//! full specification.

use std::collections::hash_map;
use std::collections::HashMap;
use std::fmt;

use aws_smithy_types::type_erasure::TypeErasedBox;
use aws_smithy_types::DiscriminatedDocument;

use crate::document::DocumentShapeDeserializer;
use crate::schema::error_envelope::sanitize_error_code;
use crate::serde::{SerdeError, ShapeDeserializer};
use crate::Schema;
use crate::ShapeId;

/// Type-erased deserialize function.
///
/// Takes a [`ShapeDeserializer`] positioned at the value to be read and
/// returns the constructed value as a [`TypeErasedBox`]. The caller
/// downcasts to recover the concrete type.
///
/// Code generation produces one of these per registered shape; the body
/// is typically a thin wrapper around the shape's generated `deserialize`
/// method that wraps the result in `TypeErasedBox::new(...)`.
pub type DeserializeFn = fn(&mut dyn ShapeDeserializer) -> Result<TypeErasedBox, SerdeError>;

/// Error-typed deserialize function.
///
/// Like [`DeserializeFn`], but returns the reified value boxed as a
/// `dyn std::error::Error` so it can be attached as the `source` of an
/// unhandled error. Error-registry entries carry one of these in addition to
/// the type-erased [`DeserializeFn`]; primary-registry entries do not.
pub type ErrorDeserializeFn =
    fn(&mut dyn ShapeDeserializer) -> Result<Box<dyn std::error::Error + Send + Sync>, SerdeError>;

/// A single entry in a [`TypeRegistry`], pairing a shape's [`Schema`]
/// with a [`DeserializeFn`] that constructs the typed value.
///
/// Error-registry entries additionally carry an [`ErrorDeserializeFn`] so the
/// reified value can be surfaced as the `source` of an unhandled error; see
/// [`reify_error`].
#[derive(Clone, Copy)]
pub struct RegistryEntry {
    schema: &'static Schema<'static>,
    deserialize: DeserializeFn,
    error_deserialize: Option<ErrorDeserializeFn>,
}

impl RegistryEntry {
    /// Build a `RegistryEntry` from a static schema reference and a
    /// type-erased deserialize function.
    pub const fn new(schema: &'static Schema<'static>, deserialize: DeserializeFn) -> Self {
        Self {
            schema,
            deserialize,
            error_deserialize: None,
        }
    }

    /// Build a `RegistryEntry` for an `@error` shape, carrying both a
    /// type-erased [`DeserializeFn`] and an [`ErrorDeserializeFn`] that boxes
    /// the reified value as a `dyn std::error::Error`.
    pub const fn new_error(
        schema: &'static Schema<'static>,
        deserialize: DeserializeFn,
        error_deserialize: ErrorDeserializeFn,
    ) -> Self {
        Self {
            schema,
            deserialize,
            error_deserialize: Some(error_deserialize),
        }
    }

    /// The static schema for this entry's shape.
    pub fn schema(&self) -> &'static Schema<'static> {
        self.schema
    }

    /// The type-erased deserialize function for this entry.
    pub fn deserialize_fn(&self) -> DeserializeFn {
        self.deserialize
    }

    /// The error deserialize function for this entry, if it was registered as
    /// an `@error` shape.
    pub fn error_deserialize_fn(&self) -> Option<ErrorDeserializeFn> {
        self.error_deserialize
    }
}

impl fmt::Debug for RegistryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The DeserializeFn pointer is not useful to print; show the shape id.
        f.debug_struct("RegistryEntry")
            .field("shape_id", self.schema.shape_id())
            .finish_non_exhaustive()
    }
}

/// A runtime mapping of [`ShapeId`] to [`RegistryEntry`].
///
/// Backed by a `HashMap<ShapeId<'static>, RegistryEntry>`. Construct via
/// [`TypeRegistry::builder`] for the most common pattern, or
/// [`TypeRegistry::new`]/[`Default::default`] for an empty registry.
///
/// Use [`TypeRegistry::compose`] to merge two registries; entries in the
/// argument registry override entries with the same shape ID in `self`.
///
/// # Cross-lifetime lookup
///
/// Internal storage is keyed by `ShapeId<'static>` (codegen-emitted shapes
/// always have static lifetime). The public `schema_for` / `entry_for`
/// methods accept `&ShapeId<'_>` of any lifetime; lookup is by fully
/// qualified name via `ShapeId`'s `Borrow<str>` impl. The companion
/// `*_fqn` methods take `&str` directly — handy after extracting a
/// `__type` field from a wire-format JSON document.
#[derive(Default)]
pub struct TypeRegistry {
    entries: HashMap<ShapeId<'static>, RegistryEntry>,
}

impl TypeRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start building a registry. See [`TypeRegistryBuilder`].
    pub fn builder() -> TypeRegistryBuilder {
        TypeRegistryBuilder {
            entries: HashMap::new(),
        }
    }

    /// Look up the static schema for `id`, or `None` if not registered.
    ///
    /// Accepts a [`ShapeId`] of any lifetime — lookup is by fully
    /// qualified name. `ShapeId<'static>` (codegen-emitted) and a
    /// runtime-built `ShapeId<'_>` with the same FQN resolve to the
    /// same entry.
    pub fn schema_for(&self, id: &ShapeId<'_>) -> Option<&'static Schema<'static>> {
        self.entries.get(id.as_str()).map(|e| e.schema)
    }

    /// Look up the static schema for the given fully qualified name
    /// (e.g. `"smithy.example#Bird"`).
    ///
    /// Use this when you already have the FQN as a string slice — for
    /// instance, after extracting a `__type` discriminator from a
    /// JSON document.
    ///
    /// # Example
    ///
    /// ```
    /// use aws_smithy_schema::prelude;
    /// use aws_smithy_schema::registry::TypeRegistry;
    /// use aws_smithy_schema::serde::{SerdeError, ShapeDeserializer};
    /// use aws_smithy_types::type_erasure::TypeErasedBox;
    ///
    /// fn deserialize_string(d: &mut dyn ShapeDeserializer) -> Result<TypeErasedBox, SerdeError> {
    ///     Ok(TypeErasedBox::new(d.read_string(&prelude::STRING)?))
    /// }
    ///
    /// let registry = TypeRegistry::builder()
    ///     .insert_shape(&prelude::STRING, deserialize_string)
    ///     .build();
    ///
    /// // FQN extracted at runtime — for instance from a wire-format
    /// // `__type` field — looks up against the static registry.
    /// assert!(registry.schema_for_fqn("smithy.api#String").is_some());
    /// assert!(registry.schema_for_fqn("smithy.api#Boolean").is_none());
    /// ```
    pub fn schema_for_fqn(&self, fqn: &str) -> Option<&'static Schema<'static>> {
        self.entries.get(fqn).map(|e| e.schema)
    }

    /// Look up the entry (schema + deserialize fn) for `id`, or `None`
    /// if not registered.
    ///
    /// Accepts a [`ShapeId`] of any lifetime; see [`Self::schema_for`].
    pub fn entry_for(&self, id: &ShapeId<'_>) -> Option<&RegistryEntry> {
        self.entries.get(id.as_str())
    }

    /// Look up the entry for the given fully qualified name. See
    /// [`Self::schema_for_fqn`].
    pub fn entry_for_fqn(&self, fqn: &str) -> Option<&RegistryEntry> {
        self.entries.get(fqn)
    }

    /// Look up the entry for an error shape by its **wire error code**
    /// (e.g. `"NotFound"`), rather than by fully qualified name.
    ///
    /// Error responses identify the error by a bare code on the wire — the
    /// `<Code>` element for restXml, the `__type` discriminator for the JSON
    /// protocols, the `x-amzn-query-error` header for awsQueryCompatible — not
    /// by a fully qualified [`ShapeId`]. This resolves such a code to its entry
    /// so a schema-serde error path can reify an error by the identifier it
    /// actually receives off the wire.
    ///
    /// `wire_code` is sanitized first (a leading `namespace#` prefix and a
    /// trailing `:url` suffix are stripped), so a bare name, a fully qualified
    /// name, or a URL-suffixed code all resolve correctly. Matching is by the
    /// registered shape's name; per-operation and per-service error sets are
    /// small, so a linear scan is sufficient and needs no extra generated index.
    ///
    /// Returns `None` if no registered shape's name matches the code.
    //
    // TODO(schema-error): honor an `@awsQueryError` code override once `Schema`
    // carries that trait — awsQueryCompatible services can emit a custom error
    // code that differs from the shape name, which this name-based match would miss.
    pub fn entry_for_error_code(&self, wire_code: &str) -> Option<&RegistryEntry> {
        let wire_code = sanitize_error_code(wire_code);
        self.iter()
            .find_map(|(id, entry)| (id.shape_name() == wire_code).then_some(entry))
    }

    /// Deserialize the given `document` into the shape pointed to by its
    /// discriminator.
    ///
    /// Returns `Err(SerdeError::InvalidInput { .. })` if the document has no
    /// discriminator, and `Err(SerdeError::UnknownMember { .. })` if the
    /// discriminator does not match a registered shape.
    ///
    /// On success, the returned [`TypeErasedBox`] carries an instance of the
    /// concrete data type that the registered [`DeserializeFn`] produces.
    /// Callers downcast via [`TypeErasedBox::downcast`].
    ///
    /// The document's lifetime is independent of the registry's storage —
    /// runtime-parsed documents (e.g. with a `__type` discriminator
    /// borrowed from input bytes) resolve against the `'static`-keyed
    /// registry via FQN matching.
    ///
    /// # Example
    ///
    /// ```
    /// use aws_smithy_schema::prelude;
    /// use aws_smithy_schema::registry::TypeRegistry;
    /// use aws_smithy_schema::serde::{SerdeError, ShapeDeserializer};
    /// use aws_smithy_types::{DiscriminatedDocument, Document};
    /// use aws_smithy_types::type_erasure::TypeErasedBox;
    ///
    /// fn deserialize_string(d: &mut dyn ShapeDeserializer) -> Result<TypeErasedBox, SerdeError> {
    ///     Ok(TypeErasedBox::new(d.read_string(&prelude::STRING)?))
    /// }
    ///
    /// let registry = TypeRegistry::builder()
    ///     .insert_shape(&prelude::STRING, deserialize_string)
    ///     .build();
    ///
    /// // Build a discriminated document. In practice this typically comes
    /// // from `JsonDeserializer::read_discriminated_document` after a
    /// // wire-format `__type` lift.
    /// let doc = DiscriminatedDocument::new(Document::String("hi".to_string()))
    ///     .with_discriminator("smithy.api#String");
    ///
    /// let typed = registry.deserialize_document(&doc).unwrap();
    /// let value = typed.downcast::<String>().unwrap();
    /// assert_eq!(*value, "hi");
    /// ```
    pub fn deserialize_document(
        &self,
        document: &DiscriminatedDocument,
    ) -> Result<TypeErasedBox, SerdeError> {
        let id = document.discriminator().ok_or_else(|| {
            SerdeError::invalid_input(
                "document has no discriminator; cannot resolve shape via TypeRegistry".to_string(),
            )
        })?;
        let entry = self
            .entries
            .get(id)
            .ok_or_else(|| SerdeError::unknown_member(id.to_string()))?;
        let mut deser = DocumentShapeDeserializer::new_with_settings(
            document.document(),
            document.settings().cloned(),
        );
        (entry.deserialize)(&mut deser)
    }

    /// Combine `self` and `other` into a new registry containing the union
    /// of their entries.
    ///
    /// On shape ID collision, entries from `other` override entries in
    /// `self`. This matches the typical use case where the caller wants
    /// extension-package entries to take precedence over base-package
    /// entries.
    pub fn compose(mut self, other: TypeRegistry) -> TypeRegistry {
        for (id, entry) in other.entries {
            self.entries.insert(id, entry);
        }
        self
    }

    /// Borrow `self` and `fallback` into a [`ComposedRegistry`] whose lookups
    /// consult `self` first and fall back to `fallback`.
    ///
    /// Unlike [`compose`](Self::compose), this neither consumes nor copies
    /// either registry — both are borrowed in place, so two `&'static`
    /// registries (the typical case for codegen-emitted statics) can be
    /// layered per call with no allocation.
    ///
    /// The precedence — `self` wins over `fallback` on a colliding lookup — is
    /// what makes this useful for **operation-scoped** error resolution: layer
    /// an operation's own error registry over the service-wide one so the
    /// operation's modeled errors take priority before widening to the rest of
    /// the service closure.
    pub fn or<'a>(&'a self, fallback: &'a TypeRegistry) -> ComposedRegistry<'a> {
        ComposedRegistry {
            primary: self,
            fallback,
        }
    }

    /// Iterate the registry's `(ShapeId, RegistryEntry)` pairs. Iteration
    /// order is unspecified.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.entries.iter(),
        }
    }

    /// Number of registered entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` iff the registry has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl fmt::Debug for TypeRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeRegistry")
            .field("entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}

/// Two [`TypeRegistry`] references layered for lookup, produced by
/// [`TypeRegistry::or`].
///
/// Lookups consult the `primary` registry first and fall back to the
/// `fallback` registry, so an entry in `primary` shadows an entry with the
/// same key in `fallback`. Both registries are borrowed, so a `ComposedRegistry`
/// is a cheap, allocation-free view — typically a per-operation error registry
/// layered over the service-wide one for operation-scoped error resolution.
#[derive(Debug, Clone, Copy)]
pub struct ComposedRegistry<'a> {
    primary: &'a TypeRegistry,
    fallback: &'a TypeRegistry,
}

impl ComposedRegistry<'_> {
    /// Look up an entry by [`ShapeId`], consulting `primary` then `fallback`.
    pub fn entry_for(&self, id: &ShapeId<'_>) -> Option<&RegistryEntry> {
        self.primary
            .entry_for(id)
            .or_else(|| self.fallback.entry_for(id))
    }

    /// Look up an entry by fully qualified name, consulting `primary` then
    /// `fallback`.
    pub fn entry_for_fqn(&self, fqn: &str) -> Option<&RegistryEntry> {
        self.primary
            .entry_for_fqn(fqn)
            .or_else(|| self.fallback.entry_for_fqn(fqn))
    }

    /// Look up an entry by wire error code, consulting `primary` then
    /// `fallback`. See [`TypeRegistry::entry_for_error_code`].
    ///
    /// This is the operation-scoped error resolution primitive: with an
    /// operation's error registry as `primary` and the service-wide registry
    /// as `fallback`, an error the operation models is resolved against the
    /// operation's own set before the lookup widens to the rest of the service.
    pub fn entry_for_error_code(&self, wire_code: &str) -> Option<&RegistryEntry> {
        self.primary
            .entry_for_error_code(wire_code)
            .or_else(|| self.fallback.entry_for_error_code(wire_code))
    }
}

/// Reify an error response body into a boxed `dyn std::error::Error`, scoped to
/// an operation and then widened to the service.
///
/// Given a `registry` composed as `operation_errors.or(&service_errors)`, a wire
/// `error_code`, and a [`ShapeDeserializer`] positioned at the error body (such
/// as the one returned by a protocol's `deserialize_error_response`), this looks
/// the code up — the operation's own errors first, then the service-wide set —
/// and runs the matched entry's [`ErrorDeserializeFn`]. The result is the typed
/// error boxed as a `dyn std::error::Error`, ready to be attached as the
/// `source` of an unhandled error.
///
/// It is the registry-backed fallback for the schema-serde error path, meant to
/// run only after an operation's modeled-error match misses, to recover
/// structure from an error the operation does not model but the service (or an
/// extension package) does. Deserialization failures are swallowed (returning
/// `None`) so a malformed or unexpected body can never change how an operation
/// fails relative to the plain generic-error path.
///
/// Body-only deserialization is correct here: the reified value is body data
/// which carries no HTTP header or status bindings, so the entry's body
/// [`ErrorDeserializeFn`] is sufficient — there is no lossy header/status case
/// to account for.
///
/// Returns `None` if `error_code` matches no registered shape, if the matched
/// entry carries no [`ErrorDeserializeFn`] (i.e. it was not registered as an
/// `@error` shape), or if deserialization of the matched shape fails.
pub fn reify_error(
    registry: ComposedRegistry<'_>,
    error_code: &str,
    deser: &mut dyn ShapeDeserializer,
) -> Option<Box<dyn std::error::Error + Send + Sync>> {
    let entry = registry.entry_for_error_code(error_code)?;
    (entry.error_deserialize_fn()?)(deser).ok()
}

/// Builder for [`TypeRegistry`].
#[derive(Default)]
pub struct TypeRegistryBuilder {
    entries: HashMap<ShapeId<'static>, RegistryEntry>,
}

impl fmt::Debug for TypeRegistryBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeRegistryBuilder")
            .field("entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}

impl TypeRegistryBuilder {
    /// Insert an explicit `(ShapeId, RegistryEntry)` pair.
    pub fn insert(mut self, id: ShapeId<'static>, entry: RegistryEntry) -> Self {
        self.entries.insert(id, entry);
        self
    }

    /// Insert a registration derived from a schema and a deserialize
    /// function. The shape ID is taken from the schema.
    ///
    /// This is the form generated code typically uses.
    pub fn insert_shape(
        mut self,
        schema: &'static Schema<'static>,
        deserialize: DeserializeFn,
    ) -> Self {
        let id = schema.shape_id().clone();
        self.entries
            .insert(id, RegistryEntry::new(schema, deserialize));
        self
    }

    /// Insert a registration for an `@error` shape, carrying both a type-erased
    /// [`DeserializeFn`] and an [`ErrorDeserializeFn`]. The shape ID is taken
    /// from the schema.
    ///
    /// This is the form generated code uses for error registries; the
    /// [`ErrorDeserializeFn`] lets [`reify_error`] surface the reified value as
    /// the `source` of an unhandled error.
    pub fn insert_error_shape(
        mut self,
        schema: &'static Schema<'static>,
        deserialize: DeserializeFn,
        error_deserialize: ErrorDeserializeFn,
    ) -> Self {
        let id = schema.shape_id().clone();
        self.entries.insert(
            id,
            RegistryEntry::new_error(schema, deserialize, error_deserialize),
        );
        self
    }

    /// Finalize the builder into a [`TypeRegistry`].
    pub fn build(self) -> TypeRegistry {
        TypeRegistry {
            entries: self.entries,
        }
    }
}

/// Iterator over the entries of a [`TypeRegistry`]. Returned by
/// [`TypeRegistry::iter`].
pub struct Iter<'a> {
    inner: hash_map::Iter<'a, ShapeId<'static>, RegistryEntry>,
}

impl fmt::Debug for Iter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Iter").finish_non_exhaustive()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a ShapeId<'static>, &'a RegistryEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for Iter<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use aws_smithy_types::{Blob, Document, DocumentError, DocumentSettings, Number};

    use std::sync::Arc;

    use crate::shape_id;
    use crate::ShapeType;

    // -- Schemas used across tests --------------------------------------------------------------

    static M_FOO_NAME: Schema<'static> = Schema::new_member(
        shape_id!("smithy.example", "Foo", "name"),
        ShapeType::String,
        "name",
        0,
    );
    static FOO_SCHEMA: Schema<'static> = Schema::new_struct(
        shape_id!("smithy.example", "Foo"),
        ShapeType::Structure,
        &[&M_FOO_NAME],
    );

    static M_BAR_VALUE: Schema<'static> = Schema::new_member(
        shape_id!("smithy.example", "Bar", "value"),
        ShapeType::Integer,
        "value",
        0,
    );
    static BAR_SCHEMA: Schema<'static> = Schema::new_struct(
        shape_id!("smithy.example", "Bar"),
        ShapeType::Structure,
        &[&M_BAR_VALUE],
    );

    // A structure with a `blob` member, used to verify that protocol
    // settings are threaded through `deserialize_document` so wire-encoded
    // blobs (JSON base64 strings) coerce on the type-registry path.
    static M_WIDGET_DATA: Schema<'static> = Schema::new_member(
        shape_id!("smithy.example", "Widget", "data"),
        ShapeType::Blob,
        "data",
        0,
    );
    static WIDGET_SCHEMA: Schema<'static> = Schema::new_struct(
        shape_id!("smithy.example", "Widget"),
        ShapeType::Structure,
        &[&M_WIDGET_DATA],
    );

    // -- Test assertion helpers -----------------------------------------------------------------

    /// Assert that `actual` is the same function as `expected`.
    ///
    /// This is a no-op under Miri. Miri does not guarantee that a function has a
    /// single unique address, so two separate reifications of the same `fn` item
    /// may compare unequal and `fn_addr_eq` would spuriously fail. Gating only
    /// the comparison (rather than `#[cfg_attr(miri, ignore)]` on the whole test)
    /// keeps the surrounding assertions running under Miri.
    fn assert_same_fn(actual: DeserializeFn, expected: DeserializeFn) {
        #[cfg(not(miri))]
        assert!(
            std::ptr::fn_addr_eq(actual, expected),
            "expected the registered deserialize fn"
        );
        #[cfg(miri)]
        let _ = (actual, expected);
    }

    // -- Test types and their deserialize fns ---------------------------------------------------

    #[derive(Debug, PartialEq)]
    struct Foo {
        name: Option<String>,
    }

    fn deserialize_foo(deser: &mut dyn ShapeDeserializer) -> Result<TypeErasedBox, SerdeError> {
        let mut out = Foo { name: None };
        deser.read_struct(&FOO_SCHEMA, &mut |member, sub| {
            if let Some(0) = member.member_index() {
                out.name = Some(sub.read_string(member)?);
            }
            Ok(())
        })?;
        Ok(TypeErasedBox::new(out))
    }

    #[derive(Debug, PartialEq)]
    struct Bar {
        value: Option<i32>,
    }

    fn deserialize_bar(deser: &mut dyn ShapeDeserializer) -> Result<TypeErasedBox, SerdeError> {
        let mut out = Bar { value: None };
        deser.read_struct(&BAR_SCHEMA, &mut |member, sub| {
            if let Some(0) = member.member_index() {
                out.value = Some(sub.read_integer(member)?);
            }
            Ok(())
        })?;
        Ok(TypeErasedBox::new(out))
    }

    // An alternative deserialize fn for Foo that returns a different type, used to
    // verify `compose` override semantics.
    #[derive(Debug, PartialEq)]
    struct FooReplacement {
        replaced: bool,
    }

    fn deserialize_foo_replacement(
        _deser: &mut dyn ShapeDeserializer,
    ) -> Result<TypeErasedBox, SerdeError> {
        Ok(TypeErasedBox::new(FooReplacement { replaced: true }))
    }

    #[derive(Debug, PartialEq)]
    struct Widget {
        data: Option<Blob>,
    }

    fn deserialize_widget(deser: &mut dyn ShapeDeserializer) -> Result<TypeErasedBox, SerdeError> {
        let mut out = Widget { data: None };
        deser.read_struct(&WIDGET_SCHEMA, &mut |member, sub| {
            if let Some(0) = member.member_index() {
                out.data = Some(sub.read_blob(member)?);
            }
            Ok(())
        })?;
        Ok(TypeErasedBox::new(out))
    }

    // A minimal JSON-like settings impl that base64-decodes strings into
    // blob bytes, the way the JSON codec does on the wire. Used to prove
    // that `deserialize_document` threads protocol settings into the
    // deserializer so blob/timestamp coercion works on the registry path.
    #[derive(Debug)]
    struct JsonishSettings;

    impl DocumentSettings for JsonishSettings {
        fn protocol_id(&self) -> &str {
            "smithy.example#Jsonish"
        }

        fn coerce_string_to_blob(&self, s: &str) -> Result<Vec<u8>, DocumentError> {
            aws_smithy_types::base64::decode(s)
                .map_err(|e| DocumentError::unsupported(format!("invalid base64 blob: {e}")))
        }
    }

    // -- Lookup tests ---------------------------------------------------------------------------

    #[test]
    fn schema_for_returns_registered_schema() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .insert_shape(&BAR_SCHEMA, deserialize_bar)
            .build();

        let foo = registry
            .schema_for(&shape_id!("smithy.example", "Foo"))
            .unwrap();
        assert_eq!(foo.shape_id(), FOO_SCHEMA.shape_id());

        let bar = registry
            .schema_for(&shape_id!("smithy.example", "Bar"))
            .unwrap();
        assert_eq!(bar.shape_id(), BAR_SCHEMA.shape_id());
    }

    #[test]
    fn schema_for_returns_none_for_unregistered_shape() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();

        assert!(registry
            .schema_for(&shape_id!("smithy.example", "DoesNotExist"))
            .is_none());
    }

    #[test]
    fn entry_for_returns_schema_and_fn() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();

        let entry = registry
            .entry_for(&shape_id!("smithy.example", "Foo"))
            .unwrap();
        assert_eq!(entry.schema().shape_id(), FOO_SCHEMA.shape_id());
        // Function pointer comparison: same fn we registered.
        assert_same_fn(entry.deserialize_fn(), deserialize_foo as DeserializeFn);
    }

    #[test]
    fn entry_for_error_code_resolves_sanitized_codes() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .insert_shape(&BAR_SCHEMA, deserialize_bar)
            .build();

        // A bare wire code, a fully qualified code, a URL-suffixed code, and a
        // fully qualified + URL-suffixed code all resolve to the same entry.
        for code in [
            "Foo",
            "smithy.example#Foo",
            "Foo:http://internal.example.com/x",
            "smithy.example#Foo:http://internal.example.com/x",
        ] {
            let entry = registry
                .entry_for_error_code(code)
                .unwrap_or_else(|| panic!("expected a match for {code:?}"));
            assert_eq!(entry.schema().shape_id(), FOO_SCHEMA.shape_id());
        }

        // The scan selects the correct entry among several.
        let bar = registry.entry_for_error_code("Bar").unwrap();
        assert_eq!(bar.schema().shape_id(), BAR_SCHEMA.shape_id());
    }

    #[test]
    fn entry_for_error_code_returns_none_for_unknown_code() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();

        assert!(registry.entry_for_error_code("DoesNotExist").is_none());
        // A namespace that happens to share the name's text must not match on
        // the namespace component — matching is on the shape name only.
        assert!(registry.entry_for_error_code("Foo#Nope").is_none());
    }

    #[test]
    fn len_and_is_empty() {
        let empty = TypeRegistry::new();
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());

        let populated = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .insert_shape(&BAR_SCHEMA, deserialize_bar)
            .build();
        assert_eq!(populated.len(), 2);
        assert!(!populated.is_empty());
    }

    #[test]
    fn iter_yields_all_entries() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .insert_shape(&BAR_SCHEMA, deserialize_bar)
            .build();

        let mut ids: Vec<String> = registry.iter().map(|(id, _)| id.to_string()).collect();
        ids.sort();
        assert_eq!(
            ids,
            vec![
                "smithy.example#Bar".to_string(),
                "smithy.example#Foo".to_string(),
            ]
        );
        // ExactSizeIterator: len() agrees with registry len.
        assert_eq!(registry.iter().len(), 2);
    }

    // -- Composition tests ----------------------------------------------------------------------

    #[test]
    fn compose_unions_disjoint_registries() {
        let a = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();
        let b = TypeRegistry::builder()
            .insert_shape(&BAR_SCHEMA, deserialize_bar)
            .build();

        let merged = a.compose(b);
        assert_eq!(merged.len(), 2);
        assert!(merged
            .schema_for(&shape_id!("smithy.example", "Foo"))
            .is_some());
        assert!(merged
            .schema_for(&shape_id!("smithy.example", "Bar"))
            .is_some());
    }

    #[test]
    fn compose_lets_other_override_self() {
        // `a` registers `Foo` → deserialize_foo (returns Foo)
        // `b` registers `Foo` → deserialize_foo_replacement (returns FooReplacement)
        // After compose, `Foo` should resolve to the replacement.
        let a = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();
        let b = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo_replacement)
            .build();

        let merged = a.compose(b);
        assert_eq!(merged.len(), 1);

        // Resolve Foo and round-trip through deserialize_document.
        let doc = DiscriminatedDocument::new(Document::Object(Default::default()))
            .with_discriminator(FOO_SCHEMA.shape_id().as_str());
        let boxed = merged.deserialize_document(&doc).unwrap();
        let result = *boxed.downcast::<FooReplacement>().expect("override fn ran");
        assert_eq!(result, FooReplacement { replaced: true });
    }

    #[test]
    fn composed_registry_prefers_primary_then_widens() {
        // `primary` models Foo (deserialize_foo); `fallback` also has Foo but
        // with a different fn, plus Bar. On the Foo collision the primary must
        // win (the opposite precedence of `compose`, where the argument wins),
        // and the lookup must widen to the fallback for Bar.
        let primary = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();
        let fallback = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo_replacement)
            .insert_shape(&BAR_SCHEMA, deserialize_bar)
            .build();
        let composed = primary.or(&fallback);

        // Collision: primary wins — the entry carries deserialize_foo, not the
        // fallback's replacement fn.
        let foo = composed.entry_for_error_code("Foo").unwrap();
        assert_same_fn(foo.deserialize_fn(), deserialize_foo as DeserializeFn);

        // Widen: Bar resolves from the fallback.
        let bar = composed.entry_for_error_code("Bar").unwrap();
        assert_eq!(bar.schema().shape_id(), BAR_SCHEMA.shape_id());

        // Miss in both: None.
        assert!(composed.entry_for_error_code("Nope").is_none());
    }

    #[test]
    fn composed_registry_entry_for_and_fqn_compose() {
        let primary = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();
        let fallback = TypeRegistry::builder()
            .insert_shape(&BAR_SCHEMA, deserialize_bar)
            .build();
        let composed = primary.or(&fallback);

        // entry_for (by ShapeId): primary hit, then fallback widen.
        assert_eq!(
            composed
                .entry_for(&shape_id!("smithy.example", "Foo"))
                .unwrap()
                .schema()
                .shape_id(),
            FOO_SCHEMA.shape_id()
        );
        assert_eq!(
            composed
                .entry_for(&shape_id!("smithy.example", "Bar"))
                .unwrap()
                .schema()
                .shape_id(),
            BAR_SCHEMA.shape_id()
        );

        // entry_for_fqn: same composition, plus a miss in both.
        assert!(composed.entry_for_fqn("smithy.example#Foo").is_some());
        assert!(composed.entry_for_fqn("smithy.example#Bar").is_some());
        assert!(composed.entry_for_fqn("smithy.example#Nope").is_none());
    }

    // Error-constructor stubs for `reify_error` tests. They ignore the
    // deserializer and return a boxed error whose `Display` identifies which
    // registry the entry came from, so the tests can assert scoping/precedence.
    fn error_ctor_primary(
        _deser: &mut dyn ShapeDeserializer,
    ) -> Result<Box<dyn std::error::Error + Send + Sync>, SerdeError> {
        Ok("primary".into())
    }

    fn error_ctor_fallback(
        _deser: &mut dyn ShapeDeserializer,
    ) -> Result<Box<dyn std::error::Error + Send + Sync>, SerdeError> {
        Ok("fallback".into())
    }

    // An error constructor that always fails, used to prove `reify_error`
    // swallows deserialize errors.
    fn error_ctor_always_errors(
        _deser: &mut dyn ShapeDeserializer,
    ) -> Result<Box<dyn std::error::Error + Send + Sync>, SerdeError> {
        Err(SerdeError::custom("boom"))
    }

    #[test]
    fn reify_error_scopes_to_primary_then_widens() {
        // `primary` models Foo (→ "primary"); `fallback` models Bar (→ "fallback").
        // Each entry carries the input-ignoring type-erased stub plus an error
        // constructor, so the test exercises reify_error's plumbing and scoping
        // rather than any specific body shape.
        let primary = TypeRegistry::builder()
            .insert_error_shape(&FOO_SCHEMA, deserialize_foo_replacement, error_ctor_primary)
            .build();
        let fallback = TypeRegistry::builder()
            .insert_error_shape(
                &BAR_SCHEMA,
                deserialize_foo_replacement,
                error_ctor_fallback,
            )
            .build();
        let composed = primary.or(&fallback);
        let doc = Document::Object(Default::default());

        // Operation-scoped hit (primary).
        let boxed = reify_error(composed, "Foo", &mut DocumentShapeDeserializer::new(&doc))
            .expect("Foo reifies from the operation registry");
        assert_eq!(boxed.to_string(), "primary");

        // Widen to the service registry for a code the operation does not model.
        let boxed = reify_error(composed, "Bar", &mut DocumentShapeDeserializer::new(&doc))
            .expect("Bar widens to the service registry");
        assert_eq!(boxed.to_string(), "fallback");

        // Unknown code in both: None.
        assert!(reify_error(composed, "Nope", &mut DocumentShapeDeserializer::new(&doc)).is_none());
    }

    #[test]
    fn reify_error_swallows_deserialize_failure() {
        // The code resolves, but the matched entry's error constructor errors —
        // reify_error must return None rather than propagate, so a malformed
        // unmodeled error can never change how the operation fails.
        let primary = TypeRegistry::builder()
            .insert_error_shape(
                &FOO_SCHEMA,
                deserialize_foo_replacement,
                error_ctor_always_errors,
            )
            .build();
        let empty = TypeRegistry::new();
        let composed = primary.or(&empty);
        let doc = Document::Object(Default::default());

        assert!(reify_error(composed, "Foo", &mut DocumentShapeDeserializer::new(&doc)).is_none());
    }

    // -- deserialize_document tests -------------------------------------------------------------

    #[test]
    fn deserialize_document_round_trip() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .insert_shape(&BAR_SCHEMA, deserialize_bar)
            .build();

        // Build a Foo document with a "name" member and the Foo discriminator.
        let mut foo_members = aws_smithy_types::document::DocumentObject::new();
        foo_members.insert("name".to_string(), Document::String("hello".to_string()));
        let foo_doc = DiscriminatedDocument::new(Document::Object(foo_members))
            .with_discriminator(FOO_SCHEMA.shape_id().as_str());

        let boxed = registry.deserialize_document(&foo_doc).unwrap();
        let foo = *boxed.downcast::<Foo>().expect("downcast to Foo");
        assert_eq!(
            foo,
            Foo {
                name: Some("hello".to_string())
            }
        );

        // Same for Bar.
        let mut bar_members = aws_smithy_types::document::DocumentObject::new();
        bar_members.insert("value".to_string(), Document::Number(Number::PosInt(42)));
        let bar_doc = DiscriminatedDocument::new(Document::Object(bar_members))
            .with_discriminator(BAR_SCHEMA.shape_id().as_str());

        let boxed = registry.deserialize_document(&bar_doc).unwrap();
        let bar = *boxed.downcast::<Bar>().expect("downcast to Bar");
        assert_eq!(bar, Bar { value: Some(42) });
    }

    #[test]
    fn deserialize_document_errors_when_discriminator_missing() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();

        // No discriminator attached.
        let doc = DiscriminatedDocument::new(Document::Object(Default::default()));

        let err = registry.deserialize_document(&doc).unwrap_err();
        match err {
            SerdeError::InvalidInput { message } => {
                assert!(
                    message.contains("discriminator"),
                    "expected message to mention discriminator, got: {message}"
                );
            }
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn deserialize_document_errors_when_shape_id_unregistered() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();

        let doc = DiscriminatedDocument::new(Document::Object(Default::default()))
            .with_discriminator("smithy.example#Unregistered");

        let err = registry.deserialize_document(&doc).unwrap_err();
        match err {
            SerdeError::UnknownMember { member_name } => {
                assert_eq!(member_name, "smithy.example#Unregistered");
            }
            other => panic!("expected UnknownMember, got {other:?}"),
        }
    }

    #[test]
    fn deserialize_document_propagates_inner_error() {
        // Foo expects `name` to be a String; supplying an integer will trigger a
        // type-mismatch from the deserialize fn.
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();

        let mut foo_members = aws_smithy_types::document::DocumentObject::new();
        foo_members.insert("name".to_string(), Document::Number(Number::PosInt(5)));
        let doc = DiscriminatedDocument::new(Document::Object(foo_members))
            .with_discriminator(FOO_SCHEMA.shape_id().as_str());

        let err = registry.deserialize_document(&doc).unwrap_err();
        match err {
            SerdeError::TypeMismatch { .. } => {}
            other => panic!("expected TypeMismatch, got {other:?}"),
        }
    }

    #[test]
    fn deserialize_document_threads_settings_for_blob_coercion() {
        let registry = TypeRegistry::builder()
            .insert_shape(&WIDGET_SCHEMA, deserialize_widget)
            .build();

        // JSON-origin document: the blob arrives as a base64 string and the
        // attached settings know how to decode it. This mirrors what
        // `JsonDeserializer::read_discriminated_document` produces. The fix
        // threads these settings into the deserializer so the nested
        // `read_blob` can coerce.
        let mut members = aws_smithy_types::document::DocumentObject::new();
        members.insert("data".to_string(), Document::String("YWJjZA==".to_string()));
        let doc = DiscriminatedDocument::new(Document::Object(members))
            .with_discriminator(WIDGET_SCHEMA.shape_id().as_str())
            .with_settings(Arc::new(JsonishSettings));

        let boxed = registry
            .deserialize_document(&doc)
            .expect("blob coercion should succeed when settings are threaded through");
        let widget = *boxed.downcast::<Widget>().expect("downcast to Widget");
        assert_eq!(
            widget,
            Widget {
                data: Some(Blob::new(b"abcd".to_vec())),
            }
        );
    }

    #[test]
    fn deserialize_document_without_settings_cannot_coerce_blob() {
        // Same base64-string blob, but no settings attached. Without
        // settings the deserializer cannot coerce a JSON string into a blob
        // and must report a type mismatch. This pins the behavior the
        // settings-threading fix depends on: it is the failure mode that
        // occurred on the registry path before the fix.
        let registry = TypeRegistry::builder()
            .insert_shape(&WIDGET_SCHEMA, deserialize_widget)
            .build();

        let mut members = aws_smithy_types::document::DocumentObject::new();
        members.insert("data".to_string(), Document::String("YWJjZA==".to_string()));
        let doc = DiscriminatedDocument::new(Document::Object(members))
            .with_discriminator(WIDGET_SCHEMA.shape_id().as_str());

        let err = registry.deserialize_document(&doc).unwrap_err();
        assert!(
            matches!(err, SerdeError::TypeMismatch { .. }),
            "expected TypeMismatch without settings, got {err:?}"
        );
    }

    // -- Builder + Debug tests ------------------------------------------------------------------

    #[test]
    fn builder_insert_keyed_form() {
        // Verify that the keyed `insert(id, entry)` form works alongside `insert_shape`.
        let id = FOO_SCHEMA.shape_id();
        let registry = TypeRegistry::builder()
            .insert(id.clone(), RegistryEntry::new(&FOO_SCHEMA, deserialize_foo))
            .build();

        assert_eq!(registry.len(), 1);
        assert!(registry.schema_for(id).is_some());
    }

    #[test]
    fn debug_impls_do_not_panic() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();
        let entry = registry.entry_for(FOO_SCHEMA.shape_id()).unwrap();

        let _ = format!("{registry:?}");
        let _ = format!("{entry:?}");
    }

    // -- Cross-lifetime lookup tests ----------------------------------------------------------

    #[test]
    fn schema_for_works_with_runtime_shape_id() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();

        // Build a runtime ShapeId<'_> with the same FQN as a registered
        // 'static shape. The registry should resolve it via the
        // FQN-based `Borrow<str>` lookup.
        let owned_fqn = String::from("smithy.example#Foo");
        let owned_ns = String::from("smithy.example");
        let owned_name = String::from("Foo");
        let runtime_id: ShapeId<'_> = ShapeId::from_parts(&owned_fqn, &owned_ns, &owned_name);

        let schema = registry.schema_for(&runtime_id).expect("found via FQN");
        assert_eq!(schema.shape_id(), FOO_SCHEMA.shape_id());

        // Negative case: same lifetime, different FQN.
        let owned_other = String::from("smithy.example#Nope");
        let runtime_miss: ShapeId<'_> = ShapeId::from_parts(&owned_other, &owned_ns, &owned_name);
        assert!(registry.schema_for(&runtime_miss).is_none());
    }

    #[test]
    fn schema_for_fqn_returns_registered_schema() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .insert_shape(&BAR_SCHEMA, deserialize_bar)
            .build();

        let foo = registry
            .schema_for_fqn("smithy.example#Foo")
            .expect("registered");
        assert_eq!(foo.shape_id(), FOO_SCHEMA.shape_id());

        assert!(registry.schema_for_fqn("smithy.example#Missing").is_none());
    }

    #[test]
    fn entry_for_fqn_returns_registered_entry() {
        let registry = TypeRegistry::builder()
            .insert_shape(&FOO_SCHEMA, deserialize_foo)
            .build();

        let entry = registry
            .entry_for_fqn("smithy.example#Foo")
            .expect("registered");
        assert_same_fn(entry.deserialize_fn(), deserialize_foo as DeserializeFn);
    }
}

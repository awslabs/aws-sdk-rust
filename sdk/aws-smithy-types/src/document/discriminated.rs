/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Schema-aware wrapper around [`Document`].
//!
//! [`DiscriminatedDocument`] adds two pieces of context that the bare
//! `Document` data type deliberately doesn't carry:
//!
//! - An optional **discriminator** — the Smithy fully-qualified shape
//!   ID of the type the document was produced from (or is intended to
//!   deserialize as). This drives the type-registry path: a JSON
//!   `__type` field gets lifted into the discriminator slot during
//!   wire parsing, and the type registry uses it to dispatch to the
//!   right schema.
//!
//! - Optional **protocol settings** — a
//!   [`DocumentSettings`] trait object
//!   describing how the source protocol encodes Smithy types that
//!   don't have native wire representations. Used by the format-aware
//!   accessors [`as_blob`](DiscriminatedDocument::as_blob) and
//!   [`as_timestamp`](DiscriminatedDocument::as_timestamp) to coerce
//!   JSON-side base64 strings into bytes, ISO-8601 strings into
//!   timestamps, etc.
//!
//! Why split this from `Document`? Two reasons:
//!
//! 1. The bare `Document` is the everyday user-facing type — it
//!    appears on operation input/output struct fields,
//!    `AuthSchemeEndpointConfig::as_document`, and in user code
//!    constructing values. Pattern matching, builders, and round-trip
//!    semantics on it should stay simple. A user holding an
//!    `Option<Document>` doesn't need to think about
//!    discriminators.
//! 2. The schema-serde pipeline does need both pieces of context, and
//!    the cleanest place for it is right here on the wrapper. The
//!    type registry's `deserialize_document` flow consumes
//!    `&DiscriminatedDocument`; codec deserializers produce
//!    `DiscriminatedDocument` (with discriminator lifted from
//!    `__type` and settings attached); type-typed shape construction
//!    via `Document::from_struct` returns a `DiscriminatedDocument`
//!    too.

use std::borrow::Cow;
use std::sync::Arc;

use crate::{DateTime, Document, DocumentError, DocumentSettings};

/// A [`Document`] together with an optional discriminator and
/// optional protocol settings.
///
/// See the module-level documentation for the rationale behind
/// splitting this off `Document`.
///
/// `#[non_exhaustive]` matches `Document`'s policy — future fields
/// (e.g. a typed `Schema` reference if the schema crate ever adds
/// schema-binding to this wrapper) can land additively.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct DiscriminatedDocument {
    /// The wrapped document data. Always present.
    document: Document,
    /// The fully-qualified shape ID of the source type, if known.
    /// Lifted from `__type` on the wire, set by
    /// [`Document::from_struct`](crate::Document) callers, or left
    /// `None` for documents constructed directly from data.
    discriminator: Option<String>,
    /// Protocol-specific settings used by format-aware coercion. Set
    /// by codec deserializers (e.g. JSON's
    /// `read_discriminated_document`), left `None` on user-built
    /// documents.
    settings: Option<Arc<dyn DocumentSettings>>,
}

impl DiscriminatedDocument {
    /// Creates a new `DiscriminatedDocument` wrapping `document`,
    /// with no discriminator and no settings attached.
    pub fn new(document: Document) -> Self {
        Self {
            document,
            discriminator: None,
            settings: None,
        }
    }

    /// Attaches a discriminator (a Smithy fully-qualified shape ID)
    /// to this document.
    ///
    /// Used by the schema-serde pipeline when constructing a
    /// document from a typed shape: the schema's `shape_id` (in its
    /// `namespace#name` FQN form) gets attached as the discriminator
    /// so downstream consumers (the type registry, the `__type`
    /// write path) know what shape the document represents.
    ///
    /// The discriminator MUST be an absolute shape ID — the
    /// `namespace#name` FQN form, never a bare shape name. The SEP
    /// requires a serialized `__type` to always be absolute so the
    /// document stays context-free for downstream readers, and the
    /// type registry keys on the FQN. Passing a relative id is a
    /// caller error and trips a `debug_assert!`.
    pub fn with_discriminator(mut self, fqn: impl Into<String>) -> Self {
        let fqn = fqn.into();
        debug_assert!(
            fqn.contains('#'),
            "discriminator `{fqn}` must be an absolute shape id (namespace#name)"
        );
        self.discriminator = Some(fqn);
        self
    }

    /// Attaches protocol settings to this document.
    ///
    /// Used by codec deserializers to plumb format-specific coercion
    /// rules through to downstream consumers of the document tree.
    /// The same `Arc` is cloned into nested documents so the entire
    /// tree shares one settings instance.
    pub fn with_settings(mut self, settings: Arc<dyn DocumentSettings>) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Sets the discriminator **without** the absolute-shape-id check
    /// that [`with_discriminator`](Self::with_discriminator) enforces.
    ///
    /// Test-only escape hatch (behind the `test-util` feature) so that
    /// downstream crates can construct a document carrying a relative
    /// (non-absolute) discriminator to exercise their own guards
    /// against one reaching the wire. Production code MUST use
    /// [`with_discriminator`](Self::with_discriminator), which
    /// guarantees the absolute `namespace#name` form the SEP requires.
    #[cfg(feature = "test-util")]
    #[doc(hidden)]
    pub fn set_discriminator_unchecked(&mut self, discriminator: impl Into<String>) {
        self.discriminator = Some(discriminator.into());
    }

    /// Returns the discriminator, if attached.
    pub fn discriminator(&self) -> Option<&str> {
        self.discriminator.as_deref()
    }

    /// Returns a reference to the attached protocol settings, if any.
    pub fn settings(&self) -> Option<&Arc<dyn DocumentSettings>> {
        self.settings.as_ref()
    }

    /// Returns a reference to the wrapped document data.
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Consumes this wrapper and returns the inner document.
    pub fn into_document(self) -> Document {
        self.document
    }

    /// Returns this document's value as bytes.
    ///
    /// Dispatches three ways:
    /// - For [`Document::Blob`], returns the bytes directly as
    ///   `Cow::Borrowed`.
    /// - For [`Document::String`], if protocol settings are attached,
    ///   delegates to
    ///   [`DocumentSettings::coerce_string_to_blob`] (typically
    ///   base64-decode for JSON) and returns `Cow::Owned`.
    /// - Otherwise returns
    ///   [`DocumentError::TypeMismatch`] (for non-blob, non-string
    ///   variants) or
    ///   [`DocumentError::UnsupportedOperation`] (for `String` with
    ///   no settings to drive the coercion).
    pub fn as_blob(&self) -> Result<Cow<'_, [u8]>, DocumentError> {
        match &self.document {
            Document::Blob(b) => Ok(Cow::Borrowed(b.as_slice())),
            Document::String(s) => match &self.settings {
                Some(settings) => settings.coerce_string_to_blob(s).map(Cow::Owned),
                None => Err(DocumentError::unsupported(
                    "cannot coerce string to blob without protocol-specific document settings",
                )),
            },
            other => Err(DocumentError::type_mismatch(format!(
                "expected blob, found {}",
                document_variant_name(other)
            ))),
        }
    }

    /// Returns this document's value as a timestamp.
    ///
    /// Dispatches four ways:
    /// - For [`Document::Timestamp`], returns the value directly.
    /// - For [`Document::String`], if settings are attached,
    ///   delegates to
    ///   [`DocumentSettings::coerce_string_to_timestamp`] (typically
    ///   parses an RFC-3339 string for JSON's default `date-time`
    ///   format).
    /// - For [`Document::Number`], if settings are attached,
    ///   delegates to
    ///   [`DocumentSettings::coerce_number_to_timestamp`] (typically
    ///   interprets the value as epoch seconds).
    /// - Otherwise returns [`DocumentError::TypeMismatch`] or
    ///   [`DocumentError::UnsupportedOperation`].
    pub fn as_timestamp(&self) -> Result<DateTime, DocumentError> {
        match (&self.document, &self.settings) {
            (Document::Timestamp(t), _) => Ok(*t),
            (Document::String(s), Some(settings)) => settings.coerce_string_to_timestamp(s),
            (Document::Number(n), Some(settings)) => settings.coerce_number_to_timestamp(n),
            (Document::String(_), None) | (Document::Number(_), None) => {
                Err(DocumentError::unsupported(
                    "cannot coerce string/number to timestamp without protocol-specific document \
                     settings",
                ))
            }
            (other, _) => Err(DocumentError::type_mismatch(format!(
                "expected timestamp, found {}",
                document_variant_name(other)
            ))),
        }
    }
}

/// `PartialEq` is implemented manually to compare `document` and
/// `discriminator` only — `settings` is metadata about how the
/// document was produced (and `dyn DocumentSettings` doesn't admit
/// equality anyway).
///
/// Two discriminated documents holding the same data with different
/// settings are considered equal: this matches the behavior of the
/// schema-crate type that this design replaces, and matches user
/// intent ("are these the same document?" doesn't depend on protocol
/// metadata).
impl PartialEq for DiscriminatedDocument {
    fn eq(&self, other: &Self) -> bool {
        self.document == other.document && self.discriminator == other.discriminator
    }
}

impl From<Document> for DiscriminatedDocument {
    fn from(document: Document) -> Self {
        Self::new(document)
    }
}

/// Returns the human-readable name of a [`Document`] variant for use
/// in error messages.
//
// Defined here, scoped to the discriminated module, because it's the
// only error-producing site that names variants. The numeric coercion
// path in `mod.rs` has its own `type_mismatch_for` for the same job
// — they're intentionally not shared because the call sites construct
// errors through different code paths and inlining the variant-name
// match is cheaper than a cross-module call.
fn document_variant_name(d: &Document) -> &'static str {
    match d {
        Document::Null => "null",
        Document::Bool(_) => "boolean",
        Document::Number(_) => "number",
        Document::String(_) => "string",
        Document::Blob(_) => "blob",
        Document::Timestamp(_) => "timestamp",
        Document::BigInteger(_) => "bigInteger",
        Document::BigDecimal(_) => "bigDecimal",
        Document::Array(_) => "array",
        Document::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    //! Tests for the `DiscriminatedDocument` wrapper and the
    //! `DocumentSettings` dispatch path.
    //!
    //! `TestSettings` is a minimal mock implementation of
    //! `DocumentSettings` — it doesn't try to do anything realistic
    //! (the JSON codec's settings will base64-decode, parse RFC-3339
    //! timestamps, etc.). The point is to exercise dispatch and
    //! error-path coverage; realistic implementations live in the
    //! codec crates that pair with this type.

    use super::*;
    use crate::Number;

    #[derive(Debug)]
    struct TestSettings {
        protocol: String,
    }

    impl DocumentSettings for TestSettings {
        fn protocol_id(&self) -> &str {
            &self.protocol
        }

        fn coerce_string_to_blob(&self, s: &str) -> Result<Vec<u8>, DocumentError> {
            // Mock: just return the bytes of the string itself.
            // A real impl would base64-decode for JSON.
            Ok(s.as_bytes().to_vec())
        }

        fn coerce_string_to_timestamp(&self, _s: &str) -> Result<DateTime, DocumentError> {
            // Mock: always return epoch.
            Ok(DateTime::from_secs(0))
        }

        fn coerce_number_to_timestamp(&self, n: &Number) -> Result<DateTime, DocumentError> {
            let secs = match n {
                Number::PosInt(v) => *v as i64,
                Number::NegInt(v) => *v,
                Number::Float(v) => *v as i64,
            };
            Ok(DateTime::from_secs(secs))
        }
    }

    fn test_settings() -> Arc<dyn DocumentSettings> {
        Arc::new(TestSettings {
            protocol: "com.example#Test".to_owned(),
        })
    }

    // -- Constructors and accessors -------------------------------------

    #[test]
    fn new_attaches_no_discriminator_or_settings() {
        let d = DiscriminatedDocument::new(Document::String("hi".to_owned()));
        assert_eq!(d.discriminator(), None);
        assert!(d.settings().is_none());
        assert_eq!(d.document().as_string(), Some("hi"));
    }

    #[test]
    fn with_discriminator_attaches_fqn() {
        let d =
            DiscriminatedDocument::new(Document::Null).with_discriminator("com.example#MyShape");
        assert_eq!(d.discriminator(), Some("com.example#MyShape"));
    }

    // The discriminator must be an absolute shape id; a bare name is a
    // caller error and trips the debug-build assertion.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "absolute shape id")]
    fn with_discriminator_rejects_relative_id() {
        let _ = DiscriminatedDocument::new(Document::Null).with_discriminator("RelativeOnly");
    }

    #[test]
    fn with_settings_attaches_settings() {
        let d = DiscriminatedDocument::new(Document::String("x".to_owned()))
            .with_settings(test_settings());
        assert!(d.settings().is_some());
        assert_eq!(d.settings().unwrap().protocol_id(), "com.example#Test");
    }

    #[test]
    fn into_document_unwraps_to_inner() {
        let inner = Document::String("hi".to_owned());
        let d = DiscriminatedDocument::new(inner.clone()).with_discriminator("com.example#X");
        assert_eq!(d.into_document(), inner);
    }

    #[test]
    fn from_document_blanket_impl_works() {
        let d: DiscriminatedDocument = Document::Bool(true).into();
        assert_eq!(d.document().as_bool(), Some(true));
        assert_eq!(d.discriminator(), None);
    }

    // -- Equality ignores settings --------------------------------------

    #[test]
    fn partial_eq_compares_document_and_discriminator_only() {
        let a = DiscriminatedDocument::new(Document::String("x".to_owned()))
            .with_discriminator("com.example#A");
        let b = DiscriminatedDocument::new(Document::String("x".to_owned()))
            .with_discriminator("com.example#A")
            .with_settings(test_settings());
        // Different settings (none vs Some), but same data + same
        // discriminator: equal.
        assert_eq!(a, b);

        let c = DiscriminatedDocument::new(Document::String("x".to_owned()))
            .with_discriminator("com.example#B");
        // Different discriminator: NOT equal.
        assert_ne!(a, c);

        let d = DiscriminatedDocument::new(Document::String("y".to_owned()))
            .with_discriminator("com.example#A");
        // Different data: NOT equal.
        assert_ne!(a, d);
    }

    // -- as_blob dispatch -----------------------------------------------

    #[test]
    fn as_blob_returns_borrowed_for_native_blob_variant() {
        let d = DiscriminatedDocument::new(Document::Blob(b"hi".to_vec()));
        match d.as_blob().unwrap() {
            Cow::Borrowed(bytes) => assert_eq!(bytes, b"hi"),
            Cow::Owned(_) => panic!("expected Cow::Borrowed for native Blob"),
        }
    }

    #[test]
    fn as_blob_native_works_without_settings_attached() {
        // Native blobs don't need settings — the variant directly
        // satisfies the request.
        let d = DiscriminatedDocument::new(Document::Blob(b"hi".to_vec()));
        assert!(d.settings().is_none());
        assert!(d.as_blob().is_ok());
    }

    #[test]
    fn as_blob_coerces_string_when_settings_present() {
        let d = DiscriminatedDocument::new(Document::String("hello".to_owned()))
            .with_settings(test_settings());
        match d.as_blob().unwrap() {
            Cow::Owned(bytes) => assert_eq!(bytes, b"hello"),
            Cow::Borrowed(_) => panic!("expected Cow::Owned for coerced String"),
        }
    }

    #[test]
    fn as_blob_string_without_settings_is_unsupported_operation() {
        let d = DiscriminatedDocument::new(Document::String("hello".to_owned()));
        let err = d.as_blob().unwrap_err();
        assert!(matches!(err, DocumentError::UnsupportedOperation { .. }));
    }

    #[test]
    fn as_blob_type_mismatch_for_non_blob_non_string_variants() {
        // Numeric variant: TypeMismatch regardless of settings.
        let d = DiscriminatedDocument::new(Document::Number(Number::PosInt(42)))
            .with_settings(test_settings());
        let err = d.as_blob().unwrap_err();
        assert!(matches!(err, DocumentError::TypeMismatch { .. }));
    }

    // -- as_timestamp dispatch ------------------------------------------

    #[test]
    fn as_timestamp_returns_direct_for_native_timestamp_variant() {
        let ts = DateTime::from_secs(1234);
        let d = DiscriminatedDocument::new(Document::Timestamp(ts));
        assert_eq!(d.as_timestamp().unwrap(), ts);
    }

    #[test]
    fn as_timestamp_native_works_without_settings() {
        // Native timestamps don't need settings either.
        let ts = DateTime::from_secs(1);
        let d = DiscriminatedDocument::new(Document::Timestamp(ts));
        assert!(d.settings().is_none());
        assert_eq!(d.as_timestamp().unwrap(), ts);
    }

    #[test]
    fn as_timestamp_coerces_string_with_settings() {
        let d = DiscriminatedDocument::new(Document::String("any string".to_owned()))
            .with_settings(test_settings());
        // Mock returns epoch.
        assert_eq!(d.as_timestamp().unwrap(), DateTime::from_secs(0));
    }

    #[test]
    fn as_timestamp_coerces_number_with_settings() {
        let d = DiscriminatedDocument::new(Document::Number(Number::PosInt(1234)))
            .with_settings(test_settings());
        assert_eq!(d.as_timestamp().unwrap(), DateTime::from_secs(1234));
    }

    #[test]
    fn as_timestamp_string_without_settings_is_unsupported() {
        let d = DiscriminatedDocument::new(Document::String("ignored".to_owned()));
        let err = d.as_timestamp().unwrap_err();
        assert!(matches!(err, DocumentError::UnsupportedOperation { .. }));
    }

    #[test]
    fn as_timestamp_number_without_settings_is_unsupported() {
        let d = DiscriminatedDocument::new(Document::Number(Number::PosInt(0)));
        let err = d.as_timestamp().unwrap_err();
        assert!(matches!(err, DocumentError::UnsupportedOperation { .. }));
    }

    #[test]
    fn as_timestamp_type_mismatch_for_non_coercible_variant() {
        // Boolean can never coerce to timestamp regardless of settings.
        let d = DiscriminatedDocument::new(Document::Bool(true)).with_settings(test_settings());
        let err = d.as_timestamp().unwrap_err();
        assert!(matches!(err, DocumentError::TypeMismatch { .. }));
    }

    // -- Default trait method bodies emit UnsupportedOperation ----------

    #[test]
    fn default_settings_methods_return_unsupported_operation() {
        // A minimal impl that only sets `protocol_id` should fall
        // through to defaults that produce UnsupportedOperation. This
        // is the path CBOR-style protocols will rely on.
        #[derive(Debug)]
        struct MinimalSettings;
        impl DocumentSettings for MinimalSettings {
            fn protocol_id(&self) -> &str {
                "com.example#Minimal"
            }
        }

        let s = MinimalSettings;
        assert!(matches!(
            s.coerce_string_to_blob("anything"),
            Err(DocumentError::UnsupportedOperation { .. })
        ));
        assert!(matches!(
            s.coerce_string_to_timestamp("anything"),
            Err(DocumentError::UnsupportedOperation { .. })
        ));
        assert!(matches!(
            s.coerce_number_to_timestamp(&Number::PosInt(0)),
            Err(DocumentError::UnsupportedOperation { .. })
        ));
    }
}

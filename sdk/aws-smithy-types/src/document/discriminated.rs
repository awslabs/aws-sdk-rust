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
//!    via `DiscriminatedDocumentExt::from_struct` (in
//!    `aws-smithy-schema`) returns a `DiscriminatedDocument` too.

use std::borrow::Cow;
use std::sync::Arc;

use crate::date_time::Format;
use crate::document::document_variant_name;
use crate::{DateTime, Document, DocumentError, DocumentSettings, Number};

/// A [`Document`] together with an optional discriminator and
/// optional protocol settings.
///
/// See the module-level documentation for the rationale behind
/// splitting this off `Document`.
///
/// This type is `#[non_exhaustive]` so that future fields (e.g. a typed
/// `Schema` reference, if the schema crate ever adds schema-binding to
/// this wrapper) can land additively.
///
/// Note that this is deliberately *unlike* [`Document`], which is
/// intentionally **not** `#[non_exhaustive]`: `Document` is a released,
/// exhaustively-matchable enum, and callers are expected to `match` it
/// without a wildcard arm. Adding a variant to `Document` would be a
/// breaking change, which is precisely why the extra context this
/// wrapper carries lives here instead of on `Document` itself.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct DiscriminatedDocument {
    /// The wrapped document data. Always present.
    document: Document,
    /// The fully-qualified shape ID of the source type, if known.
    /// Lifted from `__type` on the wire, set by
    /// `DiscriminatedDocumentExt::from_struct` (in `aws-smithy-schema`)
    /// callers, or left `None` for documents constructed directly from
    /// data.
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
    /// `Document` has no native blob variant: the schema-driven legacy
    /// representation of a Smithy `blob` is a base64-encoded
    /// [`Document::String`]. This accessor therefore dispatches two
    /// ways:
    /// - With protocol settings attached, delegates to
    ///   [`DocumentSettings::coerce_string_to_blob`].
    /// - With no settings attached, falls back to the deterministic
    ///   default: standard base64 decode.
    ///
    /// Non-string variants return [`DocumentError::TypeMismatch`].
    pub fn as_blob(&self) -> Result<Cow<'_, [u8]>, DocumentError> {
        match &self.document {
            Document::String(s) => match &self.settings {
                Some(settings) => settings.coerce_string_to_blob(s).map(Cow::Owned),
                // Deterministic default: base64, the representation every
                // JSON-family protocol uses and the one the schema
                // serializer writes.
                None => crate::base64::decode(s).map(Cow::Owned).map_err(|e| {
                    DocumentError::invalid_input(format!(
                        "cannot base64-decode string as blob: {e}"
                    ))
                }),
            },
            other => Err(DocumentError::type_mismatch(format!(
                "expected blob, found {}",
                document_variant_name(other)
            ))),
        }
    }

    /// Returns this document's value as a timestamp.
    ///
    /// `Document` has no native timestamp variant: the schema-driven
    /// legacy representation of a Smithy `timestamp` is a
    /// [`Document::Number`] holding epoch seconds. This accessor
    /// dispatches four ways:
    /// - For [`Document::Number`] with settings attached, delegates to
    ///   [`DocumentSettings::coerce_number_to_timestamp`]; with no
    ///   settings, falls back to the deterministic epoch-seconds
    ///   default.
    /// - For [`Document::String`] with settings attached, delegates to
    ///   [`DocumentSettings::coerce_string_to_timestamp`]; with no
    ///   settings, falls back to parsing RFC-3339 (`date-time`), which
    ///   is Smithy's default string timestamp format.
    ///
    /// Other variants return [`DocumentError::TypeMismatch`].
    pub fn as_timestamp(&self) -> Result<DateTime, DocumentError> {
        match (&self.document, &self.settings) {
            (Document::Number(n), Some(settings)) => settings.coerce_number_to_timestamp(n),
            (Document::Number(n), None) => number_as_epoch_seconds(n),
            (Document::String(s), Some(settings)) => settings.coerce_string_to_timestamp(s),
            (Document::String(s), None) => DateTime::from_str(s, Format::DateTime).map_err(|e| {
                DocumentError::invalid_input(format!(
                    "cannot parse string as a date-time timestamp: {e}"
                ))
            }),
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

/// Interprets a [`Number`] as epoch seconds, retaining fractional
/// seconds when the wire value carried them.
///
/// This is the deterministic default used when no protocol settings are
/// attached. `epoch-seconds` is the format the schema serializer writes
/// for a `timestamp` shape in the legacy `Document` representation.
fn number_as_epoch_seconds(n: &Number) -> Result<DateTime, DocumentError> {
    match n {
        Number::PosInt(v) => i64::try_from(*v)
            .map(DateTime::from_secs)
            .map_err(|_| DocumentError::invalid_input(format!("epoch seconds {v} out of range"))),
        Number::NegInt(v) => Ok(DateTime::from_secs(*v)),
        Number::Float(v) => {
            if !v.is_finite() {
                return Err(DocumentError::invalid_input(format!(
                    "epoch seconds {v} is not finite"
                )));
            }
            Ok(DateTime::from_secs_f64(*v))
        }
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
    fn as_blob_coerces_string_when_settings_present() {
        let d = DiscriminatedDocument::new(Document::String("hello".to_owned()))
            .with_settings(test_settings());
        match d.as_blob().unwrap() {
            // TestSettings returns the raw bytes of the string.
            Cow::Owned(bytes) => assert_eq!(bytes, b"hello"),
            Cow::Borrowed(_) => panic!("expected Cow::Owned for coerced String"),
        }
    }

    #[test]
    fn as_blob_falls_back_to_base64_without_settings() {
        // No settings attached: the deterministic default is a standard
        // base64 decode, matching what the schema serializer writes.
        let d = DiscriminatedDocument::new(Document::String("YWJjZA==".to_owned()));
        assert!(d.settings().is_none());
        assert_eq!(d.as_blob().unwrap().as_ref(), b"abcd");
    }

    #[test]
    fn as_blob_invalid_base64_without_settings_is_invalid_input() {
        let d = DiscriminatedDocument::new(Document::String("not base64!!!".to_owned()));
        let err = d.as_blob().unwrap_err();
        assert!(matches!(err, DocumentError::InvalidInput { .. }));
    }

    #[test]
    fn as_blob_type_mismatch_for_non_string_variants() {
        // Numeric variant: TypeMismatch regardless of settings.
        let d = DiscriminatedDocument::new(Document::Number(Number::PosInt(42)))
            .with_settings(test_settings());
        let err = d.as_blob().unwrap_err();
        assert!(matches!(err, DocumentError::TypeMismatch { .. }));
    }

    // -- as_timestamp dispatch ------------------------------------------

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
    fn as_timestamp_number_defaults_to_epoch_seconds_without_settings() {
        // The deterministic default: a number is epoch seconds. This is
        // the legacy representation the schema serializer writes for a
        // `timestamp` shape.
        let d = DiscriminatedDocument::new(Document::Number(Number::PosInt(1234)));
        assert!(d.settings().is_none());
        assert_eq!(d.as_timestamp().unwrap(), DateTime::from_secs(1234));

        let d = DiscriminatedDocument::new(Document::Number(Number::NegInt(-5)));
        assert_eq!(d.as_timestamp().unwrap(), DateTime::from_secs(-5));
    }

    #[test]
    fn as_timestamp_retains_fractional_epoch_seconds() {
        let d = DiscriminatedDocument::new(Document::Number(Number::Float(1234.5)));
        assert_eq!(d.as_timestamp().unwrap(), DateTime::from_secs_f64(1234.5));
    }

    #[test]
    fn as_timestamp_string_defaults_to_date_time_without_settings() {
        let d = DiscriminatedDocument::new(Document::String("1970-01-01T00:00:00Z".to_owned()));
        assert_eq!(d.as_timestamp().unwrap(), DateTime::from_secs(0));
    }

    #[test]
    fn as_timestamp_malformed_string_without_settings_is_invalid_input() {
        let d = DiscriminatedDocument::new(Document::String("not a timestamp".to_owned()));
        let err = d.as_timestamp().unwrap_err();
        assert!(matches!(err, DocumentError::InvalidInput { .. }));
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

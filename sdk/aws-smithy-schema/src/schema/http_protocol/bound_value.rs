/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Deserializers for response values that are bound to a part of an HTTP message rather
//! than to the protocol body.
//!
//! Each type here reads exactly one modeled member from one location. The composite
//! response deserializer creates one per response-bound member and hands it to the
//! generated member-index consumer, which calls a single `read_*` method on it.
//!
//! # Why not `HttpStringDeserializer`
//!
//! [`crate::codec::http_string::HttpStringDeserializer`] cannot be reused for response headers: it takes
//! `&str` (mapping invalid UTF-8 to an empty string), splits on commas without regard for
//! quoting, and ignores the schema's `@timestampFormat`. The header semantics that AWS SDKs
//! have shipped for years live in `aws_smithy_runtime_api::http::header_parse`, and this
//! module is built on those primitives so the schema path and the generated legacy path
//! cannot drift.

use crate::serde::{SerdeError, ShapeDeserializer};
use crate::{Schema, ShapeType};
use aws_smithy_runtime_api::http::header_parse;
use aws_smithy_runtime_api::http::Headers;
use aws_smithy_types::date_time::Format;
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime, Document};

/// The default timestamp format for a value bound to an HTTP header.
const HEADER_TIMESTAMP_DEFAULT: Format = Format::HttpDate;

/// Converts a header [`ParseError`](header_parse::ParseError) into a [`SerdeError`].
///
/// [`SerdeError`] carries no source, so the parser's whole chain is formatted into the
/// message. Losing it would reduce, for example, a failed integer parse to "failed reading
/// a list of primitives" with no indication of which value or why. The composite adds the
/// modeled member and header name around this.
#[cold]
fn parse_failed(err: header_parse::ParseError) -> SerdeError {
    SerdeError::invalid_input(format!("{}", DisplayErrorContext(&err)))
}

/// The raw values of a single HTTP header, re-readable without allocating.
///
/// A member can need several passes over its values — a presence check, an emptiness check,
/// the parse, and a non-UTF-8 scan if the parse failed. Every `Headers` lookup by `&str`
/// re-validates and re-hashes the header name, which profiling showed to be the dominant cost
/// of a header-bound member when each pass looked the name up again. So the lookup happens
/// once, in [`Self::new`], and the overwhelmingly common single-valued header is kept as a
/// slice. Only a header repeated across several lines looks the name up again per pass, which
/// still needs no allocation.
///
/// Values are raw bytes: a non-UTF-8 value must reach the parser so it can be reported, which
/// is what PR #4868 established, and is why this does not use the `&str` accessors.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HeaderValues<'a> {
    headers: &'a Headers,
    name: &'a str,
    resolved: Resolved<'a>,
}

/// What the single lookup in [`HeaderValues::new`] found.
#[derive(Clone, Copy, Debug)]
enum Resolved<'a> {
    Absent,
    One(&'a [u8]),
    /// Two or more values; re-read from the map on each pass.
    Many,
}

/// Iterator returned by [`HeaderValues::iter`].
///
/// A dedicated two-state iterator rather than `Option::into_iter().chain(..)`, whose `next`
/// re-checks both halves on every call and showed up in profiles of header-heavy outputs.
pub(crate) enum HeaderValuesIter<'a, I> {
    One(Option<&'a [u8]>),
    Many(I),
}

impl<'a, I: Iterator<Item = &'a [u8]>> Iterator for HeaderValuesIter<'a, I> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<&'a [u8]> {
        match self {
            HeaderValuesIter::One(value) => value.take(),
            HeaderValuesIter::Many(values) => values.next(),
        }
    }
}

impl<'a> HeaderValues<'a> {
    /// Binds to the values of `name` in `headers`, looking the name up once.
    #[inline]
    pub(crate) fn new(headers: &'a Headers, name: &'a str) -> Self {
        let mut values = headers.get_all_bytes(name);
        let resolved = match (values.next(), values.next()) {
            (None, _) => Resolved::Absent,
            (Some(value), None) => Resolved::One(value),
            (Some(_), Some(_)) => Resolved::Many,
        };
        Self {
            headers,
            name,
            resolved,
        }
    }

    /// Like [`Self::new`], but looks the values up by a parsed header name, which skips
    /// re-validating the name.
    #[inline]
    pub(crate) fn by_name(headers: &'a Headers, name: &'a http::HeaderName) -> Self {
        let mut values = headers.get_all_bytes_by_name(name);
        let resolved = match (values.next(), values.next()) {
            (None, _) => Resolved::Absent,
            (Some(value), None) => Resolved::One(value),
            (Some(_), Some(_)) => Resolved::Many,
        };
        Self {
            headers,
            // Repeated values are re-read by this name. That is the uncommon case, so it
            // takes the string lookup.
            name: name.as_str(),
            resolved,
        }
    }

    /// Iterates the raw values, including any that are not valid UTF-8.
    #[inline]
    pub(crate) fn iter(&self) -> HeaderValuesIter<'a, impl Iterator<Item = &'a [u8]>> {
        match self.resolved {
            Resolved::Absent => HeaderValuesIter::One(None),
            Resolved::One(value) => HeaderValuesIter::One(Some(value)),
            Resolved::Many => HeaderValuesIter::Many(self.headers.get_all_bytes(self.name)),
        }
    }

    /// Returns true when the header carries at least one value.
    #[inline]
    pub(crate) fn is_present(&self) -> bool {
        !matches!(self.resolved, Resolved::Absent)
    }

    /// Returns true when at least one value is not valid UTF-8.
    ///
    /// This is the `NonUtf8HeaderHandling::Skip` predicate. It is deliberately a scan of
    /// *all* values rather than a classification of the parse error, because list parsing
    /// stops at its first failure: keying off that error would make the outcome depend on
    /// whether the service happened to send the unreadable value before or after a
    /// separately malformed one. See PR #4868.
    pub(crate) fn has_unreadable_value(&self) -> bool {
        self.iter().any(|v| std::str::from_utf8(v).is_err())
    }

    /// Returns true when every value is empty, so the comma tokenizer yields no items.
    ///
    /// The tokenizer's loop is `while !header.is_empty()`, so this is exactly equivalent to
    /// "parsing produces an empty `Vec`" without doing the parse. Note that a value of
    /// `"   "` is *not* empty by this test and does yield one (empty) token, matching the
    /// tokenizer.
    #[inline]
    pub(crate) fn yields_no_tokens(&self) -> bool {
        self.iter().all(|v| v.is_empty())
    }
}

/// Reads one modeled member from the raw values of one HTTP header.
///
/// # Cardinality
///
/// A member's Rust type decides which `read_*` method the generated consumer calls, and
/// that in turn decides cardinality:
///
/// - a scalar read requires the header to hold at most one item, and fails otherwise;
/// - a list read accepts any number, across both commas and repeated header lines.
///
/// # Strings are special
///
/// A string that is not `@mediaType`-tagged is read whole, without comma splitting, because
/// a string value may legitimately contain commas. Every other type is comma-delimited per
/// RFC 7230, including quoted values. This asymmetry is not incidental — it is the behavior
/// the generated path has always had.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HttpHeaderValueDeserializer<'a> {
    values: HeaderValues<'a>,
}

impl<'a> HttpHeaderValueDeserializer<'a> {
    /// Binds to the values of one header.
    #[inline]
    pub(crate) fn new(values: HeaderValues<'a>) -> Self {
        Self { values }
    }

    /// Returns true when this header is present but the member would parse to no value.
    ///
    /// The generated legacy path reports that as an absent member, so the composite must
    /// not invoke the consumer in this case; there is no way to express "no value" through
    /// a `read_*` return type.
    ///
    /// Whether it can happen depends on the parse path. A plain string is read whole, so an
    /// empty header value is the legitimate value `""`. Everything else is tokenized, and an
    /// empty value yields no tokens at all.
    #[inline]
    pub(crate) fn parses_to_no_value(&self, schema: &Schema<'_>) -> bool {
        if reads_whole_value(schema) {
            return false;
        }
        self.values.yields_no_tokens()
    }

    /// Requires that a scalar read produced a value.
    ///
    /// Unreachable when the composite honors [`Self::parses_to_no_value`]; kept as an
    /// explicit failure rather than a silent default so a future call site that forgets the
    /// check is loud instead of fabricating a zero.
    #[inline]
    fn require<T>(&self, parsed: Option<T>, expected: &str) -> Result<T, SerdeError> {
        match parsed {
            Some(value) => Ok(value),
            None => Err(self.held_no_value(expected)),
        }
    }

    #[cold]
    fn held_no_value(&self, expected: &str) -> SerdeError {
        SerdeError::invalid_input(format!(
            "expected {expected} in header `{}` but it held no value",
            self.values.name
        ))
    }

    /// Parses every comma-delimited token as a string, applying `@mediaType` decoding.
    fn string_tokens(&self, element: &Schema<'_>) -> Result<Vec<String>, SerdeError> {
        let tokens: Vec<String> =
            header_parse::read_many_from_str_bytes(self.values.iter()).map_err(parse_failed)?;
        if element.media_type().is_some() {
            tokens.into_iter().map(decode_media_type).collect()
        } else {
            Ok(tokens)
        }
    }
}

/// Returns true when a member's header value is taken whole rather than comma-split.
///
/// Only a string without `@mediaType`. A `@mediaType` string is base64, which cannot
/// contain a comma, so it takes the ordinary delimited path.
#[inline]
fn reads_whole_value(schema: &Schema<'_>) -> bool {
    schema.shape_type() == ShapeType::String && schema.media_type().is_none()
}

/// Decodes one `@mediaType` header token: base64 of the modeled string's bytes.
fn decode_media_type(token: String) -> Result<String, SerdeError> {
    let bytes = aws_smithy_types::base64::decode(&token)
        .map_err(|_| SerdeError::invalid_input("failed to decode base64"))?;
    String::from_utf8(bytes)
        .map_err(|_| SerdeError::invalid_input("base64 encoded data was not valid utf-8"))
}

impl ShapeDeserializer for HttpHeaderValueDeserializer<'_> {
    fn read_struct(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(
            &Schema<'_>,
            &mut dyn ShapeDeserializer,
        ) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "a structure cannot be bound to an HTTP header",
        ))
    }

    fn read_list(
        &mut self,
        schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // Elements are parsed up front, then handed over one at a time. They cannot be
        // parsed lazily inside the consumer: an HTTP-date's own comma means the split and
        // the parse are one operation, so the element type has to be known before
        // tokenizing, and only the container schema carries it. Generated element reads pass
        // a *prelude* schema (e.g. `prelude::TIMESTAMP`), so the element's traits must be
        // resolved here, from the container, rather than from the schema the consumer sees.
        let element = schema.member();
        let is_timestamp = element.map(|e| e.shape_type()) == Some(ShapeType::Timestamp);
        if is_timestamp {
            let format = header_timestamp_format(schema);
            let dates =
                header_parse::many_dates_bytes(self.values.iter(), format).map_err(parse_failed)?;
            for date in dates {
                consumer(&mut HeaderTokenDeserializer::date(date))?;
            }
        } else {
            let tokens = self.string_tokens(element.unwrap_or(&crate::prelude::STRING))?;
            for token in &tokens {
                consumer(&mut HeaderTokenDeserializer::text(token))?;
            }
        }
        Ok(())
    }

    fn read_map(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // A map bound to headers is `@httpPrefixHeaders`, which spans many header names and
        // therefore is not a single header's value.
        Err(SerdeError::unsupported(
            "a map cannot be bound to a single HTTP header",
        ))
    }

    fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
        let parsed =
            header_parse::one_primitive_or_none_bytes(self.values.iter()).map_err(parse_failed)?;
        self.require(parsed, "a boolean")
    }

    fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
        let parsed =
            header_parse::one_primitive_or_none_bytes(self.values.iter()).map_err(parse_failed)?;
        self.require(parsed, "a byte")
    }

    fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
        let parsed =
            header_parse::one_primitive_or_none_bytes(self.values.iter()).map_err(parse_failed)?;
        self.require(parsed, "a short")
    }

    fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
        let parsed =
            header_parse::one_primitive_or_none_bytes(self.values.iter()).map_err(parse_failed)?;
        self.require(parsed, "an integer")
    }

    fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
        let parsed =
            header_parse::one_primitive_or_none_bytes(self.values.iter()).map_err(parse_failed)?;
        self.require(parsed, "a long")
    }

    fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
        let parsed =
            header_parse::one_primitive_or_none_bytes(self.values.iter()).map_err(parse_failed)?;
        self.require(parsed, "a float")
    }

    fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
        let parsed =
            header_parse::one_primitive_or_none_bytes(self.values.iter()).map_err(parse_failed)?;
        self.require(parsed, "a double")
    }

    fn read_big_integer(&mut self, _schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
        let parsed =
            header_parse::one_from_str_or_none_bytes(self.values.iter()).map_err(parse_failed)?;
        self.require(parsed, "a big integer")
    }

    fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        let parsed =
            header_parse::one_from_str_or_none_bytes(self.values.iter()).map_err(parse_failed)?;
        self.require(parsed, "a big decimal")
    }

    fn read_string(&mut self, schema: &Schema<'_>) -> Result<String, SerdeError> {
        if reads_whole_value(schema) {
            let parsed =
                header_parse::one_or_none_bytes(self.values.iter()).map_err(parse_failed)?;
            return self.require(parsed, "a string");
        }
        // `@mediaType`: comma-delimited base64, scalar cardinality, then decode.
        let parsed: Option<String> =
            header_parse::one_from_str_or_none_bytes(self.values.iter()).map_err(parse_failed)?;
        decode_media_type(self.require(parsed, "a string")?)
    }

    fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        // Smithy forbids a blob target for `@httpHeader`, so the generated consumer never
        // reaches this for a valid model. Defined rather than rejected so the type is total
        // and so the base64 meaning of a header blob is unambiguous if it ever is reached.
        let parsed: Option<String> =
            header_parse::one_from_str_or_none_bytes(self.values.iter()).map_err(parse_failed)?;
        let encoded = self.require(parsed, "a blob")?;
        decode_blob(&encoded)
    }

    fn read_timestamp(&mut self, schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
        let format = header_timestamp_format(schema);
        let parsed = header_parse::one_date_or_none_bytes(self.values.iter(), format)
            .map_err(parse_failed)?;
        self.require(parsed, "a timestamp")
    }

    fn read_document(&mut self, _schema: &Schema<'_>) -> Result<Document, SerdeError> {
        Err(SerdeError::unsupported(
            "a document cannot be bound to an HTTP header",
        ))
    }

    /// Always false.
    ///
    /// A generated optional-member arm checks `is_null()` before its typed read and skips
    /// the member when it is true. The composite only creates this deserializer for a header
    /// that is present, so reporting null here would silently drop every optional
    /// header-bound member.
    fn is_null(&self) -> bool {
        false
    }

    /// Nothing to advance: this deserializer holds a value the composite already isolated
    /// from the message, and the composite's own iteration over bound members is what moves
    /// forward. It is also never the parent of a member consumer.
    fn skip_value(&mut self) -> Result<(), SerdeError> {
        Ok(())
    }

    fn container_size(&self) -> Option<usize> {
        // Element count is only known after tokenizing, which would mean parsing twice.
        None
    }

    fn read_string_list(&mut self, schema: &Schema<'_>) -> Result<Vec<String>, SerdeError> {
        self.string_tokens(schema.member().unwrap_or(&crate::prelude::STRING))
    }

    fn read_blob_list(&mut self, _schema: &Schema<'_>) -> Result<Vec<Blob>, SerdeError> {
        let tokens: Vec<String> =
            header_parse::read_many_from_str_bytes(self.values.iter()).map_err(parse_failed)?;
        tokens.iter().map(|t| decode_blob(t)).collect()
    }

    fn read_integer_list(&mut self, _schema: &Schema<'_>) -> Result<Vec<i32>, SerdeError> {
        header_parse::read_many_primitive_bytes(self.values.iter()).map_err(parse_failed)
    }

    fn read_long_list(&mut self, _schema: &Schema<'_>) -> Result<Vec<i64>, SerdeError> {
        header_parse::read_many_primitive_bytes(self.values.iter()).map_err(parse_failed)
    }
}

/// Resolves the timestamp format for a value bound to a header.
///
/// Reads the format from the member schema the composite is bound to, defaulting to
/// `http-date`. For a list of timestamps the format comes from the member, not the element:
/// that mirrors the generated path, which resolves the format from the structure member and
/// its target, and so does not see an element-level `@timestampFormat`.
fn header_timestamp_format(schema: &Schema<'_>) -> Format {
    super::timestamp_format_or(schema, HEADER_TIMESTAMP_DEFAULT)
}

fn decode_blob(encoded: &str) -> Result<Blob, SerdeError> {
    aws_smithy_types::base64::decode(encoded)
        .map(Blob::new)
        .map_err(|_| SerdeError::invalid_input("failed to decode base64"))
}

/// One already-extracted element of a header-bound list.
///
/// Elements are pre-parsed by [`HttpHeaderValueDeserializer::read_list`], so this only
/// converts an extracted token to the type the consumer asks for. Timestamps arrive already
/// parsed because their format is known only at the container.
#[derive(Debug)]
enum HeaderToken<'t> {
    Text(&'t str),
    Date(DateTime),
}

/// Presents a single header list element to a generated element consumer.
#[derive(Debug)]
struct HeaderTokenDeserializer<'t> {
    token: HeaderToken<'t>,
}

impl<'t> HeaderTokenDeserializer<'t> {
    fn text(token: &'t str) -> Self {
        Self {
            token: HeaderToken::Text(token),
        }
    }

    fn date(date: DateTime) -> Self {
        Self {
            token: HeaderToken::Date(date),
        }
    }

    fn text_value(&self, expected: &str) -> Result<&'t str, SerdeError> {
        match self.token {
            HeaderToken::Text(text) => Ok(text),
            HeaderToken::Date(_) => Err(SerdeError::invalid_input(format!(
                "expected {expected} but the header list element was a timestamp"
            ))),
        }
    }

    fn primitive<T: aws_smithy_types::primitive::Parse>(
        &self,
        expected: &str,
    ) -> Result<T, SerdeError> {
        let text = self.text_value(expected)?;
        T::parse_smithy_primitive(text)
            .map_err(|err| SerdeError::invalid_input(format!("{}", DisplayErrorContext(&err))))
    }

    fn parse_from_str<T: std::str::FromStr>(&self, expected: &str) -> Result<T, SerdeError>
    where
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        let text = self.text_value(expected)?;
        text.parse().map_err(|err| {
            SerdeError::invalid_input(format!(
                "failed to parse {expected}: {}",
                DisplayErrorContext(&err)
            ))
        })
    }
}

impl ShapeDeserializer for HeaderTokenDeserializer<'_> {
    fn read_struct(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(
            &Schema<'_>,
            &mut dyn ShapeDeserializer,
        ) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "a structure cannot be an element of a header-bound list",
        ))
    }

    fn read_list(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "a header-bound list cannot nest another list",
        ))
    }

    fn read_map(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "a map cannot be an element of a header-bound list",
        ))
    }

    fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
        self.primitive("a boolean")
    }

    fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
        self.primitive("a byte")
    }

    fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
        self.primitive("a short")
    }

    fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
        self.primitive("an integer")
    }

    fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
        self.primitive("a long")
    }

    fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
        self.primitive("a float")
    }

    fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
        self.primitive("a double")
    }

    fn read_big_integer(&mut self, _schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
        self.parse_from_str("a big integer")
    }

    fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        self.parse_from_str("a big decimal")
    }

    fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
        // `@mediaType` decoding already happened during tokenizing, because it applies to
        // the whole list uniformly and the element schema the consumer passes may be a
        // prelude schema carrying no traits.
        Ok(self.text_value("a string")?.to_string())
    }

    fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        decode_blob(self.text_value("a blob")?)
    }

    fn read_timestamp(&mut self, _schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
        match self.token {
            HeaderToken::Date(date) => Ok(date),
            HeaderToken::Text(_) => Err(SerdeError::invalid_input(
                "expected a timestamp but the header list was not parsed as timestamps",
            )),
        }
    }

    fn read_document(&mut self, _schema: &Schema<'_>) -> Result<Document, SerdeError> {
        Err(SerdeError::unsupported(
            "a document cannot be an element of a header-bound list",
        ))
    }

    /// Always false; see [`HttpHeaderValueDeserializer::is_null`]. A header list has no
    /// representation for a null element, so every element the consumer sees is a value.
    fn is_null(&self) -> bool {
        false
    }

    fn skip_value(&mut self) -> Result<(), SerdeError> {
        Ok(())
    }

    fn container_size(&self) -> Option<usize> {
        None
    }
}

/// Presents the HTTP response status code to a generated `@httpResponseCode` member.
///
/// The modeled type is always an integer, so this reads a value it already holds rather than
/// parsing anything. The generated legacy path assigns the status unconditionally, so the
/// composite always invokes the consumer for this member.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HttpStatusDeserializer {
    status: u16,
}

impl HttpStatusDeserializer {
    pub(crate) fn new(status: u16) -> Self {
        Self { status }
    }

    fn only_an_integer<T>(&self, requested: &str) -> Result<T, SerdeError> {
        Err(SerdeError::invalid_input(format!(
            "@httpResponseCode must target an integer, but {requested} was requested"
        )))
    }
}

impl ShapeDeserializer for HttpStatusDeserializer {
    fn read_struct(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(
            &Schema<'_>,
            &mut dyn ShapeDeserializer,
        ) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.only_an_integer("a structure")
    }

    fn read_list(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.only_an_integer("a list")
    }

    fn read_map(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.only_an_integer("a map")
    }

    fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
        self.only_an_integer("a boolean")
    }

    fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
        self.only_an_integer("a byte")
    }

    fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
        // A status fits in an i16, but Smithy's `@httpResponseCode` targets an integer, so
        // accepting a narrower read would be inventing a binding the model cannot express.
        self.only_an_integer("a short")
    }

    /// The whole purpose of this deserializer.
    fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
        Ok(self.status as i32)
    }

    fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
        self.only_an_integer("a long")
    }

    fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
        self.only_an_integer("a float")
    }

    fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
        self.only_an_integer("a double")
    }

    fn read_big_integer(&mut self, _schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
        self.only_an_integer("a big integer")
    }

    fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        self.only_an_integer("a big decimal")
    }

    fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
        self.only_an_integer("a string")
    }

    fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        self.only_an_integer("a blob")
    }

    fn read_timestamp(&mut self, _schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
        self.only_an_integer("a timestamp")
    }

    fn read_document(&mut self, _schema: &Schema<'_>) -> Result<Document, SerdeError> {
        self.only_an_integer("a document")
    }

    /// Always false; see [`HttpHeaderValueDeserializer::is_null`]. A response always has a
    /// status, so this member always has a value.
    fn is_null(&self) -> bool {
        false
    }

    fn skip_value(&mut self) -> Result<(), SerdeError> {
        Ok(())
    }

    fn container_size(&self) -> Option<usize> {
        None
    }
}

/// Reads a `@httpPrefixHeaders` map from every response header whose name carries the prefix.
///
/// # Transactionality
///
/// A partial map would silently conceal dropped entries, so this must produce either the whole
/// map or nothing. That holds because an entry failure propagates out of `read_map` /
/// `read_string_string_map` before the generated arm assigns the builder, and because the
/// composite suppresses the entire member when `NonUtf8HeaderHandling::Skip` applies.
///
/// # No matches is not the same as absent
///
/// When no header carries the prefix, the member is `Some(empty_map)`, not `None`. That is
/// existing Smithy behavior, so [`is_null`](Self::is_null) is false and the composite invokes
/// the consumer unconditionally; `read_map` then yields zero entries.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HttpPrefixHeadersDeserializer<'a> {
    headers: &'a Headers,
    prefix: &'a str,
}

impl<'a> HttpPrefixHeadersDeserializer<'a> {
    /// Binds to every header in `headers` whose name starts with `prefix`.
    pub(crate) fn new(headers: &'a Headers, prefix: &'a str) -> Self {
        Self { headers, prefix }
    }

    /// The full names of the matching headers, paired with their unprefixed keys.
    ///
    /// Matching is case-insensitive and allocates nothing; the prefix is compared in place
    /// rather than lowercased into a temporary.
    fn matches(&self) -> impl Iterator<Item = (&'a str, &'a str)> {
        header_parse::headers_for_prefix(
            self.headers.iter_bytes().map(|(name, _)| name),
            self.prefix,
        )
    }

    /// Returns true when at least one matching header holds a value that is not valid UTF-8.
    ///
    /// This is the `Skip` predicate for the whole member. Like the single-header case it scans
    /// every matching value rather than inspecting the parse error, so the decision does not
    /// depend on which entry the map iteration reached first.
    pub(crate) fn has_unreadable_value(&self) -> bool {
        self.matches()
            .any(|(_, name)| HeaderValues::new(self.headers, name).has_unreadable_value())
    }

    /// A deserializer over one matching header's values.
    fn value_of(&self, name: &'a str) -> HttpHeaderValueDeserializer<'a> {
        HttpHeaderValueDeserializer::new(HeaderValues::new(self.headers, name))
    }

    fn only_a_map<T>(&self, requested: &str) -> Result<T, SerdeError> {
        Err(SerdeError::invalid_input(format!(
            "@httpPrefixHeaders must target a map, but {requested} was requested"
        )))
    }
}

impl ShapeDeserializer for HttpPrefixHeadersDeserializer<'_> {
    fn read_struct(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(
            &Schema<'_>,
            &mut dyn ShapeDeserializer,
        ) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.only_a_map("a structure")
    }

    fn read_list(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.only_a_map("a list")
    }

    fn read_map(
        &mut self,
        _schema: &Schema<'_>,
        consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // Matching headers are streamed straight to the consumer: no intermediate collection
        // of bindings is built, and the only allocation is the key that becomes a map entry.
        // The consumer passes its own value schema to whichever `read_*` it calls, so the
        // modeled value type stays the consumer's concern.
        let this = *self;
        for (key, name) in this.matches() {
            consumer(key.to_string(), &mut this.value_of(name))?;
        }
        Ok(())
    }

    fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
        self.only_a_map("a boolean")
    }

    fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
        self.only_a_map("a byte")
    }

    fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
        self.only_a_map("a short")
    }

    fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
        self.only_a_map("an integer")
    }

    fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
        self.only_a_map("a long")
    }

    fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
        self.only_a_map("a float")
    }

    fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
        self.only_a_map("a double")
    }

    fn read_big_integer(&mut self, _schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
        self.only_a_map("a big integer")
    }

    fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        self.only_a_map("a big decimal")
    }

    fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
        self.only_a_map("a string")
    }

    fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        self.only_a_map("a blob")
    }

    fn read_timestamp(&mut self, _schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
        self.only_a_map("a timestamp")
    }

    fn read_document(&mut self, _schema: &Schema<'_>) -> Result<Document, SerdeError> {
        self.only_a_map("a document")
    }

    /// Always false. An absent prefix is `Some(empty_map)`, so the consumer must still run.
    fn is_null(&self) -> bool {
        false
    }

    fn skip_value(&mut self) -> Result<(), SerdeError> {
        Ok(())
    }

    fn container_size(&self) -> Option<usize> {
        // Counting would mean a second pass over the header map for a pre-allocation hint.
        None
    }

    /// The generated arm for a `map<string, string>` member — the only shape Smithy allows
    /// for `@httpPrefixHeaders` — calls this, so it is the hot path. Building the map here
    /// avoids a dynamic call per entry.
    fn read_string_string_map(
        &mut self,
        schema: &Schema<'_>,
    ) -> Result<std::collections::HashMap<String, String>, SerdeError> {
        // `member()` on a map schema is the *value* schema. Reading it here is what keeps the
        // modeled value's traits — `@mediaType`, for instance — in effect on this fast path.
        let value_schema = schema.member().unwrap_or(&crate::prelude::STRING);
        let this = *self;
        let mut out = std::collections::HashMap::new();
        for (key, name) in this.matches() {
            // `?` here is what makes the map transactional: `out` is dropped rather than
            // handed back partially populated.
            let value = this.value_of(name).read_string(value_schema)?;
            out.insert(key.to_string(), value);
        }
        Ok(out)
    }
}

/// Presents a non-streaming `@httpPayload` body to a generated blob or string member.
///
/// Structure, union, and document payloads are not handled here: the protocol's body codec
/// reads those, positioned at the payload root. A `@streaming` payload is not handled here
/// either — the generated streaming path owns the live body, and the composite must not invoke
/// the consumer for such a member.
///
/// # An empty body means an absent member
///
/// The generated legacy path guards the assignment with `if !body.is_empty()`, so an empty
/// payload leaves the member `None`. That rule applies to structured payloads too, so the
/// check lives in the composite's payload routing rather than here. [`is_null`](Self::is_null)
/// stays false for the same reason it does on the other bound-value deserializers.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HttpRawPayloadDeserializer<'a> {
    body: &'a [u8],
}

impl<'a> HttpRawPayloadDeserializer<'a> {
    pub(crate) fn new(body: &'a [u8]) -> Self {
        Self { body }
    }

    fn only_raw<T>(&self, requested: &str) -> Result<T, SerdeError> {
        Err(SerdeError::invalid_input(format!(
            "a raw @httpPayload is a blob or string, but {requested} was requested"
        )))
    }
}

impl ShapeDeserializer for HttpRawPayloadDeserializer<'_> {
    fn read_struct(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(
            &Schema<'_>,
            &mut dyn ShapeDeserializer,
        ) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // A structure payload is parsed by the body codec, not here.
        self.only_raw("a structure")
    }

    fn read_list(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.only_raw("a list")
    }

    fn read_map(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.only_raw("a map")
    }

    fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
        self.only_raw("a boolean")
    }

    fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
        self.only_raw("a byte")
    }

    fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
        self.only_raw("a short")
    }

    fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
        self.only_raw("an integer")
    }

    fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
        self.only_raw("a long")
    }

    fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
        self.only_raw("a float")
    }

    fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
        self.only_raw("a double")
    }

    fn read_big_integer(&mut self, _schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
        self.only_raw("a big integer")
    }

    fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        self.only_raw("a big decimal")
    }

    /// The body as a string, replacing invalid UTF-8 rather than failing.
    ///
    /// Lossy conversion is the shipped behavior of the generated path. It is deliberately not
    /// tightened here: a payload that is almost text would start failing responses that
    /// currently succeed.
    fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
        Ok(String::from_utf8_lossy(self.body).into_owned())
    }

    /// The body verbatim.
    fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        Ok(Blob::new(self.body.to_vec()))
    }

    fn read_timestamp(&mut self, _schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
        self.only_raw("a timestamp")
    }

    fn read_document(&mut self, _schema: &Schema<'_>) -> Result<Document, SerdeError> {
        // A document payload is parsed by the protocol's body codec, which knows the wire
        // format; raw bytes cannot be turned into a document without one.
        self.only_raw("a document")
    }

    /// Always false; see [`HttpHeaderValueDeserializer::is_null`]. An empty payload is an
    /// absent member, which the composite decides before invoking a consumer, rather than a
    /// null one.
    fn is_null(&self) -> bool {
        false
    }

    fn skip_value(&mut self) -> Result<(), SerdeError> {
        Ok(())
    }

    fn container_size(&self) -> Option<usize> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::TimestampFormat;
    use crate::{shape_id, ShapeType};

    /// Builds a `Headers` from raw bytes, so a value that is not valid UTF-8 can be tested.
    ///
    /// `Headers::insert`/`append` accept string-like values only, so they cannot express
    /// one. Values are appended, so repeating a name produces multiple header lines.
    fn headers(pairs: &[(&str, &[u8])]) -> Headers {
        let mut map = http::HeaderMap::new();
        for (name, value) in pairs {
            map.append(
                http::HeaderName::from_bytes(name.as_bytes()).expect("valid header name"),
                http::HeaderValue::from_bytes(value).expect("valid header value"),
            );
        }
        Headers::try_from(map).expect("valid headers")
    }

    fn deser<'a>(headers: &'a Headers, name: &'a str) -> HttpHeaderValueDeserializer<'a> {
        HttpHeaderValueDeserializer::new(HeaderValues::new(headers, name))
    }

    /// One header with one value, plus a bound deserializer over it.
    macro_rules! one {
        ($value:expr) => {
            headers(&[("x-test", $value)])
        };
    }

    const ID: crate::ShapeId<'static> = shape_id!("test", "S");

    static STRING_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::String, "name", 0).with_http_header("x-test");
    static MEDIA_MEMBER: Schema<'static> = Schema::new_member(ID, ShapeType::String, "doc", 0)
        .with_http_header("x-test")
        .with_media_type("application/json");
    static BOOL_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Boolean, "enabled", 0).with_http_header("x-test");
    static BYTE_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Byte, "b", 0).with_http_header("x-test");
    static SHORT_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Short, "s", 0).with_http_header("x-test");
    static INT_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Integer, "attempts", 0).with_http_header("x-test");
    static LONG_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Long, "l", 0).with_http_header("x-test");
    static FLOAT_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Float, "ratio", 0).with_http_header("x-test");
    static DOUBLE_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Double, "d", 0).with_http_header("x-test");
    static BIG_INT_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::BigInteger, "bi", 0).with_http_header("x-test");
    static BIG_DEC_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::BigDecimal, "bd", 0).with_http_header("x-test");
    static BLOB_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Blob, "raw", 0).with_http_header("x-test");

    static TS_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Timestamp, "at", 0).with_http_header("x-test");
    static TS_DATE_TIME_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Timestamp, "at", 0)
            .with_http_header("x-test")
            .with_timestamp_format(TimestampFormat::DateTime);
    static TS_EPOCH_MEMBER: Schema<'static> = Schema::new_member(ID, ShapeType::Timestamp, "at", 0)
        .with_http_header("x-test")
        .with_timestamp_format(TimestampFormat::EpochSeconds);

    static STRING_ELEMENT: Schema<'static> = Schema::new(shape_id!("test", "E"), ShapeType::String);
    static MEDIA_ELEMENT: Schema<'static> =
        Schema::new(shape_id!("test", "E"), ShapeType::String).with_media_type("application/json");
    static INT_ELEMENT: Schema<'static> = Schema::new(shape_id!("test", "E"), ShapeType::Integer);
    static LONG_ELEMENT: Schema<'static> = Schema::new(shape_id!("test", "E"), ShapeType::Long);
    static TS_ELEMENT: Schema<'static> = Schema::new(shape_id!("test", "E"), ShapeType::Timestamp);

    static STRING_LIST_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::List, "labels", 0)
            .with_http_header("x-test")
            .with_list_member(&STRING_ELEMENT);
    static MEDIA_LIST_MEMBER: Schema<'static> = Schema::new_member(ID, ShapeType::List, "docs", 0)
        .with_http_header("x-test")
        .with_list_member(&MEDIA_ELEMENT);
    static INT_LIST_MEMBER: Schema<'static> = Schema::new_member(ID, ShapeType::List, "codes", 0)
        .with_http_header("x-test")
        .with_list_member(&INT_ELEMENT);
    static LONG_LIST_MEMBER: Schema<'static> = Schema::new_member(ID, ShapeType::List, "sizes", 0)
        .with_http_header("x-test")
        .with_list_member(&LONG_ELEMENT);
    static TS_LIST_MEMBER: Schema<'static> = Schema::new_member(ID, ShapeType::List, "dates", 0)
        .with_http_header("x-test")
        .with_list_member(&TS_ELEMENT);
    static TS_LIST_EPOCH_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::List, "dates", 0)
            .with_http_header("x-test")
            .with_timestamp_format(TimestampFormat::EpochSeconds)
            .with_list_member(&TS_ELEMENT);

    // -- scalar strings are taken whole --

    #[test]
    fn a_scalar_string_keeps_its_commas() {
        // The reason strings do not go through the comma tokenizer: a string value may
        // legitimately contain commas, and splitting would corrupt it.
        let h = one!(b"a,b,c");
        assert_eq!(
            deser(&h, "x-test").read_string(&STRING_MEMBER).unwrap(),
            "a,b,c"
        );
    }

    #[test]
    fn a_scalar_string_is_trimmed() {
        let h = one!(b"  spaced  ");
        assert_eq!(
            deser(&h, "x-test").read_string(&STRING_MEMBER).unwrap(),
            "spaced"
        );
    }

    #[test]
    fn a_scalar_string_rejects_repeated_header_lines() {
        let h = headers(&[("x-test", b"a"), ("x-test", b"b")]);
        let err = deser(&h, "x-test")
            .read_string(&STRING_MEMBER)
            .expect_err("two values for a scalar");
        assert!(
            format!("{err}").contains("expected a single value but found multiple"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn an_empty_scalar_string_is_the_empty_string_not_an_absent_member() {
        // Unlike every other type, an empty value is a legitimate value here, so the
        // composite must not treat the member as absent.
        let h = one!(b"");
        let d = deser(&h, "x-test");
        assert!(!d.parses_to_no_value(&STRING_MEMBER));
        assert_eq!(deser(&h, "x-test").read_string(&STRING_MEMBER).unwrap(), "");
    }

    // -- scalar primitives --

    #[test]
    fn scalar_primitives_parse() {
        let h = one!(b"true");
        assert!(deser(&h, "x-test").read_boolean(&BOOL_MEMBER).unwrap());
        let h = one!(b"-7");
        assert_eq!(deser(&h, "x-test").read_byte(&BYTE_MEMBER).unwrap(), -7);
        let h = one!(b"1234");
        assert_eq!(deser(&h, "x-test").read_short(&SHORT_MEMBER).unwrap(), 1234);
        let h = one!(b"42");
        assert_eq!(deser(&h, "x-test").read_integer(&INT_MEMBER).unwrap(), 42);
        let h = one!(b"9876543210");
        assert_eq!(
            deser(&h, "x-test").read_long(&LONG_MEMBER).unwrap(),
            9876543210
        );
        let h = one!(b"0.5");
        assert_eq!(deser(&h, "x-test").read_float(&FLOAT_MEMBER).unwrap(), 0.5);
        let h = one!(b"0.25");
        assert_eq!(
            deser(&h, "x-test").read_double(&DOUBLE_MEMBER).unwrap(),
            0.25
        );
    }

    #[test]
    fn floats_accept_the_smithy_special_values() {
        // These are Smithy spellings, not Rust's: `f32::from_str` does not accept
        // "Infinity". Using the shared primitive parser rather than `FromStr` is what makes
        // them work.
        let h = one!(b"NaN");
        assert!(deser(&h, "x-test")
            .read_float(&FLOAT_MEMBER)
            .unwrap()
            .is_nan());
        let h = one!(b"Infinity");
        assert_eq!(
            deser(&h, "x-test").read_double(&DOUBLE_MEMBER).unwrap(),
            f64::INFINITY
        );
        let h = one!(b"-Infinity");
        assert_eq!(
            deser(&h, "x-test").read_float(&FLOAT_MEMBER).unwrap(),
            f32::NEG_INFINITY
        );
    }

    #[test]
    fn a_scalar_primitive_rejects_multiple_values() {
        for h in [one!(b"1,2"), headers(&[("x-test", b"1"), ("x-test", b"2")])] {
            let err = deser(&h, "x-test")
                .read_integer(&INT_MEMBER)
                .expect_err("two values for a scalar");
            assert!(
                format!("{err}").contains("expected one item but found 2"),
                "unexpected error: {err}"
            );
        }
    }

    #[test]
    fn a_malformed_primitive_is_an_error_and_keeps_its_cause() {
        let h = one!(b"notanint");
        let err = deser(&h, "x-test")
            .read_integer(&INT_MEMBER)
            .expect_err("not an integer");
        let message = format!("{err}");
        assert!(
            message.contains("failed to parse input as i32"),
            "the parser's source chain must survive into SerdeError: {message}"
        );
    }

    #[test]
    fn big_numbers_parse() {
        let h = one!(b"170141183460469231731687303715884105728");
        assert_eq!(
            deser(&h, "x-test")
                .read_big_integer(&BIG_INT_MEMBER)
                .unwrap()
                .as_ref(),
            "170141183460469231731687303715884105728"
        );
        let h = one!(b"1.0000000000000000000000001");
        assert_eq!(
            deser(&h, "x-test")
                .read_big_decimal(&BIG_DEC_MEMBER)
                .unwrap()
                .as_ref(),
            "1.0000000000000000000000001"
        );
    }

    // -- timestamps --

    #[test]
    fn a_timestamp_defaults_to_http_date() {
        // The comma inside an HTTP-date is the reason timestamps need a format-aware split
        // rather than the generic tokenizer.
        let h = one!(b"Wed, 21 Oct 2015 07:28:00 GMT");
        let parsed = deser(&h, "x-test").read_timestamp(&TS_MEMBER).unwrap();
        assert_eq!(parsed.secs(), 1_445_412_480);
    }

    #[test]
    fn a_timestamp_honors_the_schema_format() {
        let h = one!(b"2015-10-21T07:28:00Z");
        assert_eq!(
            deser(&h, "x-test")
                .read_timestamp(&TS_DATE_TIME_MEMBER)
                .unwrap()
                .secs(),
            1_445_412_480
        );
        let h = one!(b"1445412480");
        assert_eq!(
            deser(&h, "x-test")
                .read_timestamp(&TS_EPOCH_MEMBER)
                .unwrap()
                .secs(),
            1_445_412_480
        );
        // And the default really is http-date, not date-time: an http-date value fails when
        // the schema asks for date-time.
        let h = one!(b"Wed, 21 Oct 2015 07:28:00 GMT");
        assert!(deser(&h, "x-test")
            .read_timestamp(&TS_DATE_TIME_MEMBER)
            .is_err());
    }

    #[test]
    fn a_scalar_timestamp_rejects_multiple_values() {
        let h = one!(b"Wed, 21 Oct 2015 07:28:00 GMT,Wed, 21 Oct 2015 07:28:01 GMT");
        let err = deser(&h, "x-test")
            .read_timestamp(&TS_MEMBER)
            .expect_err("two dates");
        assert!(
            format!("{err}").contains("expected one item but found 2"),
            "unexpected error: {err}"
        );
    }

    // -- @mediaType --

    #[test]
    fn a_media_type_string_is_base64_decoded() {
        let h = one!(b"eyJhIjoxfQ==");
        assert_eq!(
            deser(&h, "x-test").read_string(&MEDIA_MEMBER).unwrap(),
            "{\"a\":1}"
        );
    }

    #[test]
    fn invalid_media_type_base64_is_an_error_not_a_silent_skip() {
        let h = one!(b"!!!not base64!!!");
        let err = deser(&h, "x-test")
            .read_string(&MEDIA_MEMBER)
            .expect_err("invalid base64");
        assert!(
            format!("{err}").contains("failed to decode base64"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn media_type_base64_that_decodes_to_non_utf8_is_an_error() {
        // Valid base64 whose bytes are not UTF-8. The modeled type is a string, so this
        // cannot be represented and must fail rather than be silently dropped.
        let h = one!(b"/w==");
        let err = deser(&h, "x-test")
            .read_string(&MEDIA_MEMBER)
            .expect_err("not utf-8 after decoding");
        assert!(
            format!("{err}").contains("base64 encoded data was not valid utf-8"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn a_media_type_list_decodes_each_element() {
        let h = one!(b"YQ==,Yg==");
        assert_eq!(
            deser(&h, "x-test")
                .read_string_list(&MEDIA_LIST_MEMBER)
                .unwrap(),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    // -- lists --

    #[test]
    fn a_string_list_splits_on_commas_and_across_header_lines() {
        let h = headers(&[("x-test", b"a,b"), ("x-test", b"c")]);
        assert_eq!(
            deser(&h, "x-test")
                .read_string_list(&STRING_LIST_MEMBER)
                .unwrap(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn a_string_list_honors_rfc_7230_quoting() {
        // A quoted element may contain the delimiter and escaped quotes. A naive
        // `split(',')` gets this wrong, which is why the shared parser is used.
        let h = one!(b"\"a,b\",plain,\"say \\\"hi\\\"\"");
        assert_eq!(
            deser(&h, "x-test")
                .read_string_list(&STRING_LIST_MEMBER)
                .unwrap(),
            vec![
                "a,b".to_string(),
                "plain".to_string(),
                "say \"hi\"".to_string()
            ]
        );
    }

    #[test]
    fn integer_and_long_lists_parse() {
        let h = one!(b"1, 2,3");
        assert_eq!(
            deser(&h, "x-test")
                .read_integer_list(&INT_LIST_MEMBER)
                .unwrap(),
            vec![1, 2, 3]
        );
        let h = one!(b"9876543210,1");
        assert_eq!(
            deser(&h, "x-test")
                .read_long_list(&LONG_LIST_MEMBER)
                .unwrap(),
            vec![9876543210, 1]
        );
    }

    #[test]
    fn a_generic_list_read_streams_elements_to_the_consumer() {
        let h = one!(b"a,b,c");
        let mut collected = Vec::new();
        deser(&h, "x-test")
            .read_list(&STRING_LIST_MEMBER, &mut |d| {
                // Generated element reads pass a prelude schema, so this mirrors codegen.
                collected.push(d.read_string(&crate::prelude::STRING)?);
                Ok(())
            })
            .unwrap();
        assert_eq!(collected, vec!["a", "b", "c"]);
    }

    #[test]
    fn a_timestamp_list_splits_http_dates_by_format_not_by_comma() {
        // Two HTTP-dates, each containing a comma. A comma split would produce four broken
        // tokens; the format-aware reader produces two dates.
        let h = one!(b"Wed, 21 Oct 2015 07:28:00 GMT,Wed, 21 Oct 2015 07:28:01 GMT");
        let mut collected = Vec::new();
        deser(&h, "x-test")
            .read_list(&TS_LIST_MEMBER, &mut |d| {
                collected.push(d.read_timestamp(&crate::prelude::TIMESTAMP)?);
                Ok(())
            })
            .unwrap();
        assert_eq!(
            collected.iter().map(|d| d.secs()).collect::<Vec<_>>(),
            vec![1_445_412_480, 1_445_412_481]
        );
    }

    #[test]
    fn a_timestamp_list_takes_its_format_from_the_container_member() {
        // The element schema the consumer receives is `prelude::TIMESTAMP`, which carries no
        // `@timestampFormat`. If the format were read from that argument instead of from the
        // container, this would fall back to http-date and fail.
        let h = one!(b"1445412480,1445412481");
        let mut collected = Vec::new();
        deser(&h, "x-test")
            .read_list(&TS_LIST_EPOCH_MEMBER, &mut |d| {
                collected.push(d.read_timestamp(&crate::prelude::TIMESTAMP)?);
                Ok(())
            })
            .unwrap();
        assert_eq!(
            collected.iter().map(|d| d.secs()).collect::<Vec<_>>(),
            vec![1_445_412_480, 1_445_412_481]
        );
    }

    #[test]
    fn a_list_element_error_propagates_out_of_the_list_read() {
        let h = one!(b"1,notanint");
        let err = deser(&h, "x-test")
            .read_integer_list(&INT_LIST_MEMBER)
            .expect_err("second element is malformed");
        assert!(
            format!("{err}").contains("failed to parse input as i32"),
            "unexpected error: {err}"
        );
    }

    // -- blobs: not a legal header target, but defined --

    #[test]
    fn a_blob_header_is_base64() {
        let h = one!(b"AQIDBA==");
        assert_eq!(
            deser(&h, "x-test")
                .read_blob(&BLOB_MEMBER)
                .unwrap()
                .as_ref(),
            &[1, 2, 3, 4]
        );
        let h = one!(b"AQID,BAUG");
        assert_eq!(
            deser(&h, "x-test")
                .read_blob_list(&STRING_LIST_MEMBER)
                .unwrap()
                .iter()
                .map(|b| b.as_ref().to_vec())
                .collect::<Vec<_>>(),
            vec![vec![1, 2, 3], vec![4, 5, 6]]
        );
    }

    // -- absence and emptiness --

    #[test]
    fn an_empty_value_parses_to_no_value_for_everything_except_a_plain_string() {
        let h = one!(b"");
        let d = deser(&h, "x-test");
        for schema in [
            &INT_MEMBER,
            &BOOL_MEMBER,
            &TS_MEMBER,
            &STRING_LIST_MEMBER,
            &MEDIA_MEMBER,
        ] {
            assert!(
                d.parses_to_no_value(schema),
                "{:?} must report no value",
                schema.member_name()
            );
        }
        assert!(
            !d.parses_to_no_value(&STRING_MEMBER),
            "a plain string reads the empty value as \"\""
        );
    }

    #[test]
    fn a_whitespace_only_value_does_yield_a_token() {
        // Not the same as empty: the tokenizer returns one (empty) token for whitespace, so
        // an integer member sees a malformed value rather than an absent one. Matching the
        // tokenizer here is what keeps the composite's absence check exact.
        let h = one!(b"   ");
        let d = deser(&h, "x-test");
        assert!(!d.parses_to_no_value(&INT_MEMBER));
        assert!(deser(&h, "x-test").read_integer(&INT_MEMBER).is_err());
    }

    #[test]
    fn a_list_whose_values_are_empty_reads_as_an_empty_vec() {
        // The generated legacy path reports an empty parse result as an *absent* member, so
        // the composite must consult `parses_to_no_value` rather than assigning this.
        let h = one!(b"");
        assert!(deser(&h, "x-test")
            .read_string_list(&STRING_LIST_MEMBER)
            .unwrap()
            .is_empty());
        assert!(deser(&h, "x-test").parses_to_no_value(&STRING_LIST_MEMBER));
    }

    #[test]
    fn a_scalar_read_of_a_valueless_header_fails_loudly() {
        // Unreachable when the composite honors `parses_to_no_value`; asserted so that a
        // future call site which forgets the check cannot fabricate a default.
        let h = one!(b"");
        let err = deser(&h, "x-test")
            .read_integer(&INT_MEMBER)
            .expect_err("no value");
        assert!(
            format!("{err}").contains("held no value"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn presence_is_reported_from_the_header_map() {
        let h = one!(b"v");
        assert!(HeaderValues::new(&h, "x-test").is_present());
        assert!(!HeaderValues::new(&h, "x-absent").is_present());
    }

    // -- non-UTF-8 --

    #[test]
    fn a_non_utf8_value_is_an_error_and_is_detectable_as_unreadable() {
        let h = one!(b"value-\xe9");
        let values = HeaderValues::new(&h, "x-test");
        assert!(
            values.has_unreadable_value(),
            "the raw octets must still be visible for the Skip decision"
        );
        for err in [
            deser(&h, "x-test")
                .read_string(&STRING_MEMBER)
                .expect_err("not utf-8"),
            deser(&h, "x-test")
                .read_integer(&INT_MEMBER)
                .expect_err("not utf-8"),
            deser(&h, "x-test")
                .read_timestamp(&TS_MEMBER)
                .expect_err("not utf-8"),
        ] {
            assert!(
                format!("{err}").contains("not valid utf-8"),
                "unexpected error: {err}"
            );
        }
        let err = deser(&h, "x-test")
            .read_string_list(&STRING_LIST_MEMBER)
            .expect_err("not utf-8");
        assert!(
            format!("{err}").contains("not valid utf-8"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn an_unreadable_value_is_found_regardless_of_its_position() {
        // PR #4868's order-independence rule: list parsing stops at its first failure, so
        // the Skip decision must come from scanning all raw values, not from the error.
        for pairs in [
            vec![("x-test", b"value-\xe9".as_slice()), ("x-test", b"ok")],
            vec![("x-test", b"ok".as_slice()), ("x-test", b"value-\xe9")],
        ] {
            let h = headers(&pairs);
            assert!(HeaderValues::new(&h, "x-test").has_unreadable_value());
        }
    }

    #[test]
    fn a_readable_but_malformed_value_is_not_unreadable() {
        // The distinction that keeps Skip from hiding ordinary parse failures.
        let h = one!(b"notanint");
        assert!(!HeaderValues::new(&h, "x-test").has_unreadable_value());
        assert!(deser(&h, "x-test").read_integer(&INT_MEMBER).is_err());
    }

    // -- trait contract --

    #[test]
    fn is_null_is_always_false() {
        // A generated optional-member arm checks `is_null()` first; reporting true would
        // silently drop every optional header-bound member.
        let h = one!(b"");
        assert!(!deser(&h, "x-test").is_null());
        let h = one!(b"v");
        assert!(!deser(&h, "x-test").is_null());
        assert!(!HeaderTokenDeserializer::text("v").is_null());
        assert!(!HeaderTokenDeserializer::date(DateTime::from_secs(0)).is_null());
    }

    #[test]
    fn skip_value_does_not_consume_anything() {
        let h = one!(b"v");
        let mut d = deser(&h, "x-test");
        let dynamic: &mut dyn ShapeDeserializer = &mut d;
        dynamic.skip_value().expect("skip must succeed");
        assert_eq!(
            deser(&h, "x-test").read_string(&STRING_MEMBER).unwrap(),
            "v",
            "the value must still be readable"
        );
    }

    #[test]
    fn aggregates_that_cannot_be_bound_to_one_header_are_rejected() {
        let h = one!(b"v");
        let mut d = deser(&h, "x-test");
        assert!(d.read_struct(&STRING_MEMBER, &mut |_, _| Ok(())).is_err());
        assert!(d.read_map(&STRING_MEMBER, &mut |_, _| Ok(())).is_err());
        assert!(d.read_document(&crate::prelude::DOCUMENT).is_err());
        // A prefix-header map is a different location, handled by its own deserializer.
        let err = d.read_map(&STRING_MEMBER, &mut |_, _| Ok(())).unwrap_err();
        assert!(
            format!("{err}").contains("single HTTP header"),
            "unexpected error: {err}"
        );
    }

    // ---------------------------------------------------------------------------------
    // @httpResponseCode
    // ---------------------------------------------------------------------------------

    static STATUS_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Integer, "status_code", 0).with_http_response_code();

    #[test]
    fn the_status_is_read_as_an_integer() {
        for status in [200u16, 201, 404, 503] {
            assert_eq!(
                HttpStatusDeserializer::new(status)
                    .read_integer(&STATUS_MEMBER)
                    .unwrap(),
                status as i32
            );
        }
    }

    #[test]
    fn the_status_deserializer_is_never_null_and_skips_cleanly() {
        let mut d = HttpStatusDeserializer::new(200);
        assert!(!d.is_null());
        let dynamic: &mut dyn ShapeDeserializer = &mut d;
        dynamic.skip_value().expect("skip must succeed");
    }

    #[test]
    fn the_status_rejects_reads_the_model_cannot_ask_for() {
        // `@httpResponseCode` targets an integer, so anything else is a modeling error rather
        // than something to coerce. A silent coercion would hide it.
        let mut d = HttpStatusDeserializer::new(200);
        assert!(d.read_string(&STATUS_MEMBER).is_err());
        assert!(d.read_long(&STATUS_MEMBER).is_err());
        assert!(d.read_short(&STATUS_MEMBER).is_err());
        assert!(d.read_timestamp(&STATUS_MEMBER).is_err());
        assert!(d.read_document(&crate::prelude::DOCUMENT).is_err());
        assert!(d.read_map(&STATUS_MEMBER, &mut |_, _| Ok(())).is_err());
        let err = d.read_string(&STATUS_MEMBER).unwrap_err();
        assert!(
            format!("{err}").contains("@httpResponseCode must target an integer"),
            "unexpected error: {err}"
        );
    }

    // ---------------------------------------------------------------------------------
    // @httpPrefixHeaders
    // ---------------------------------------------------------------------------------

    static PREFIX_KEY: Schema<'static> = Schema::new(shape_id!("test", "K"), ShapeType::String);
    static PREFIX_VALUE: Schema<'static> = Schema::new(shape_id!("test", "V"), ShapeType::String);
    static PREFIX_MEDIA_VALUE: Schema<'static> =
        Schema::new(shape_id!("test", "V"), ShapeType::String).with_media_type("application/json");

    static PREFIX_MEMBER: Schema<'static> = Schema::new_member(ID, ShapeType::Map, "metadata", 0)
        .with_http_prefix_headers("x-meta-")
        .with_map_members(&PREFIX_KEY, &PREFIX_VALUE);
    /// Deliberately mixed case, as a modeler would write it.
    static PREFIX_MIXED_CASE_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Map, "metadata", 0)
            .with_http_prefix_headers("X-Meta-")
            .with_map_members(&PREFIX_KEY, &PREFIX_VALUE);
    static PREFIX_MEDIA_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Map, "metadata", 0)
            .with_http_prefix_headers("x-meta-")
            .with_map_members(&PREFIX_KEY, &PREFIX_MEDIA_VALUE);

    fn prefix<'a>(headers: &'a Headers, p: &'a str) -> HttpPrefixHeadersDeserializer<'a> {
        HttpPrefixHeadersDeserializer::new(headers, p)
    }

    #[test]
    fn a_prefix_map_collects_only_matching_headers_keyed_by_the_remainder() {
        let h = headers(&[
            ("x-meta-color", b"red"),
            ("x-meta-size", b"large"),
            ("x-other", b"ignored"),
            ("content-type", b"application/json"),
        ]);
        let map = prefix(&h, "x-meta-")
            .read_string_string_map(&PREFIX_MEMBER)
            .unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("color"), Some(&"red".to_string()));
        assert_eq!(map.get("size"), Some(&"large".to_string()));
    }

    #[test]
    fn no_matching_headers_is_an_empty_map_not_an_absent_member() {
        // Existing Smithy behavior: the member is `Some(empty_map)`. That is why `is_null` is
        // false here — the composite must still invoke the consumer.
        let h = headers(&[("x-other", b"v")]);
        let mut d = prefix(&h, "x-meta-");
        assert!(!d.is_null());
        assert!(d.read_string_string_map(&PREFIX_MEMBER).unwrap().is_empty());
    }

    #[test]
    fn a_mixed_case_modeled_prefix_matches_normalized_wire_names() {
        let h = headers(&[("x-meta-color", b"red")]);
        let map = prefix(&h, "X-Meta-")
            .read_string_string_map(&PREFIX_MIXED_CASE_MEMBER)
            .unwrap();
        assert_eq!(map.get("color"), Some(&"red".to_string()));
    }

    #[test]
    fn a_prefixed_header_with_repeated_values_is_a_cardinality_error() {
        // The pre-existing schema path used `HashMap::insert` and silently kept one value.
        // Routing each header through the shared scalar parser makes this an error, matching
        // the generated legacy path.
        let h = headers(&[("x-meta-color", b"red"), ("x-meta-color", b"blue")]);
        let err = prefix(&h, "x-meta-")
            .read_string_string_map(&PREFIX_MEMBER)
            .expect_err("two values for one prefixed header");
        assert!(
            format!("{err}").contains("expected a single value but found multiple"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn a_failing_entry_yields_no_map_at_all() {
        // Transactionality: a partial map would conceal dropped entries, and there is no way
        // to observe one because the error replaces the whole result.
        let h = headers(&[
            ("x-meta-good", b"ok"),
            ("x-meta-bad", b"one"),
            ("x-meta-bad", b"two"),
        ]);
        assert!(prefix(&h, "x-meta-")
            .read_string_string_map(&PREFIX_MEMBER)
            .is_err());
        // The raw headers are untouched, so an interceptor can still see everything.
        assert_eq!(h.get_all_bytes("x-meta-bad").count(), 2);
    }

    #[test]
    fn the_prefix_map_honors_the_modeled_value_schema() {
        // `@mediaType` on the map value means base64. If the value schema were ignored, this
        // would come back as the literal base64 text.
        let h = headers(&[("x-meta-doc", b"eyJhIjoxfQ==")]);
        let map = prefix(&h, "x-meta-")
            .read_string_string_map(&PREFIX_MEDIA_MEMBER)
            .unwrap();
        assert_eq!(map.get("doc"), Some(&"{\"a\":1}".to_string()));
    }

    #[test]
    fn the_generic_map_read_agrees_with_the_string_map_fast_path() {
        let h = headers(&[("x-meta-color", b"red"), ("x-meta-size", b"large")]);
        let mut collected = std::collections::HashMap::new();
        prefix(&h, "x-meta-")
            .read_map(&PREFIX_MEMBER, &mut |key, d| {
                collected.insert(key, d.read_string(&PREFIX_VALUE)?);
                Ok(())
            })
            .unwrap();
        assert_eq!(
            collected,
            prefix(&h, "x-meta-")
                .read_string_string_map(&PREFIX_MEMBER)
                .unwrap()
        );
    }

    #[test]
    fn an_unreadable_prefixed_value_is_detectable_and_ignores_other_headers() {
        let h = headers(&[("x-meta-color", b"value-\xe9"), ("x-meta-size", b"large")]);
        assert!(prefix(&h, "x-meta-").has_unreadable_value());
        assert!(prefix(&h, "x-meta-")
            .read_string_string_map(&PREFIX_MEMBER)
            .is_err());

        // A non-UTF-8 value on a header *outside* the prefix must not make the member
        // skippable — that would let an unrelated header suppress a modeled one.
        let h = headers(&[("x-meta-color", b"red"), ("x-other", b"value-\xe9")]);
        assert!(!prefix(&h, "x-meta-").has_unreadable_value());
        assert!(prefix(&h, "x-meta-")
            .read_string_string_map(&PREFIX_MEMBER)
            .is_ok());
    }

    #[test]
    fn a_prefix_rejects_reads_the_model_cannot_ask_for() {
        let h = headers(&[("x-meta-color", b"red")]);
        let mut d = prefix(&h, "x-meta-");
        assert!(d.read_string(&PREFIX_MEMBER).is_err());
        assert!(d.read_integer(&PREFIX_MEMBER).is_err());
        assert!(d.read_list(&PREFIX_MEMBER, &mut |_| Ok(())).is_err());
        let err = d.read_string(&PREFIX_MEMBER).unwrap_err();
        assert!(
            format!("{err}").contains("@httpPrefixHeaders must target a map"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn the_prefix_deserializer_skips_cleanly() {
        let h = headers(&[("x-meta-color", b"red")]);
        let mut d = prefix(&h, "x-meta-");
        let dynamic: &mut dyn ShapeDeserializer = &mut d;
        dynamic.skip_value().expect("skip must succeed");
        assert_eq!(
            prefix(&h, "x-meta-")
                .read_string_string_map(&PREFIX_MEMBER)
                .unwrap()
                .len(),
            1
        );
    }

    // ---------------------------------------------------------------------------------
    // raw @httpPayload
    // ---------------------------------------------------------------------------------

    static BLOB_PAYLOAD_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Blob, "payload", 0).with_http_payload();
    static STRING_PAYLOAD_MEMBER: Schema<'static> =
        Schema::new_member(ID, ShapeType::String, "payload", 0).with_http_payload();

    #[test]
    fn a_blob_payload_is_the_body_verbatim() {
        let body = b"\x00\x01\xff binary";
        assert_eq!(
            HttpRawPayloadDeserializer::new(body)
                .read_blob(&BLOB_PAYLOAD_MEMBER)
                .unwrap()
                .as_ref(),
            body
        );
    }

    #[test]
    fn a_string_payload_is_lossy_rather_than_fallible() {
        // Shipped behavior. Tightening it would start failing responses that succeed today.
        assert_eq!(
            HttpRawPayloadDeserializer::new(b"hello")
                .read_string(&STRING_PAYLOAD_MEMBER)
                .unwrap(),
            "hello"
        );
        let lossy = HttpRawPayloadDeserializer::new(b"bad-\xe9")
            .read_string(&STRING_PAYLOAD_MEMBER)
            .expect("invalid utf-8 must not fail");
        assert_eq!(lossy, "bad-\u{fffd}");
    }

    #[test]
    fn an_empty_payload_reports_itself_absent_without_claiming_to_be_null() {
        let empty = HttpRawPayloadDeserializer::new(b"");
        assert!(
            !empty.is_null(),
            "absence is decided by the composite from the body length, not reported as null"
        );
        // If the composite ignored that, the value it would produce is an empty one — which is
        // exactly the divergence the check exists to prevent.
        assert!(HttpRawPayloadDeserializer::new(b"")
            .read_blob(&BLOB_PAYLOAD_MEMBER)
            .unwrap()
            .as_ref()
            .is_empty());
    }

    #[test]
    fn a_structured_or_document_payload_is_not_this_deserializers_job() {
        // Those are read by the protocol's body codec, positioned at the payload root. Raw
        // bytes cannot be interpreted without knowing the wire format.
        let mut d = HttpRawPayloadDeserializer::new(b"{}");
        assert!(d.read_document(&crate::prelude::DOCUMENT).is_err());
        assert!(d
            .read_struct(&BLOB_PAYLOAD_MEMBER, &mut |_, _| Ok(()))
            .is_err());
        assert!(d.read_integer(&BLOB_PAYLOAD_MEMBER).is_err());
        let err = d.read_document(&crate::prelude::DOCUMENT).unwrap_err();
        assert!(
            format!("{err}").contains("raw @httpPayload is a blob or string"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn the_payload_deserializer_skips_cleanly() {
        let mut d = HttpRawPayloadDeserializer::new(b"body");
        let dynamic: &mut dyn ShapeDeserializer = &mut d;
        dynamic.skip_value().expect("skip must succeed");
        assert_eq!(
            HttpRawPayloadDeserializer::new(b"body")
                .read_string(&STRING_PAYLOAD_MEMBER)
                .unwrap(),
            "body"
        );
    }

    /// Design §8.4 requires all four bound-value deserializers to report not-null, because a
    /// generated optional-member arm checks `is_null()` before its typed read and would
    /// otherwise drop every optional transport-bound member.
    #[test]
    fn every_bound_value_deserializer_reports_not_null() {
        let h = headers(&[("x-test", b""), ("x-meta-k", b"")]);
        let header_value = deser(&h, "x-test");
        let prefix_map = prefix(&h, "x-meta-");
        let status = HttpStatusDeserializer::new(204);
        let payload = HttpRawPayloadDeserializer::new(b"");
        let sources: [&dyn ShapeDeserializer; 4] = [&header_value, &prefix_map, &status, &payload];
        for (index, source) in sources.iter().enumerate() {
            assert!(
                !source.is_null(),
                "bound-value deserializer {index} must not report null"
            );
        }
    }
}

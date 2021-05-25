/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! XML Encoding module that uses Rust lifetimes to make
//! generating malformed XML a compile error

use crate::escape::escape;
use std::fmt::{self, Display, Formatter, Write};

// currently there's actually no way that encoding can fail but give it time :-)
#[derive(Debug)]
pub enum Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Xml Encoding Error")
    }
}

/// XmlWriter Abstraction
///
/// XmlWriter (and friends) make generating an invalid XML document a type error. Nested branches
/// of the Xml document mutable borrow from the root. You cannot continue writing to the root
/// until the nested branch is dropped and dropping the nested branch writes the terminator (eg.
/// closing element).
///
/// The one exception to this rule is names—it is possible to construct an invalid Xml Name. However,
/// names are always known ahead of time and always static, so this would be obvious from the code.
///
/// Furthermore, once `const panic` stabilizes, we'll be able to make an invalid XmlName a compiler
/// error.
///
/// ## Example
/// ```rust
/// use smithy_xml::encode::XmlWriter;
/// let mut s = String::new();
/// let mut doc = XmlWriter::new(&mut s);
/// let mut start_el = doc.start_el("Root")
///     .write_ns("http://example.com", None);
/// let mut start_tag = start_el.finish();
/// start_tag.data("hello");
/// start_tag.finish();
/// assert_eq!(s, "<Root xmlns=\"http://example.com\">hello</Root>");
/// ```
///
/// See `tests/handwritten_serializers.rs` for more usage examples.
pub struct XmlWriter<'a> {
    doc: &'a mut String,
}

impl<'a> XmlWriter<'a> {
    pub fn new(doc: &'a mut String) -> Self {
        Self { doc }
    }
}

impl<'a> XmlWriter<'a> {
    pub fn start_el<'b, 'c>(&'c mut self, tag: &'b str) -> ElWriter<'c, 'b> {
        write!(self.doc, "<{}", tag).unwrap();
        ElWriter {
            doc: self.doc,
            start: tag,
        }
    }
}

pub struct ElWriter<'a, 'b> {
    start: &'b str,
    doc: &'a mut String,
}

impl<'a, 'b> ElWriter<'a, 'b> {
    pub fn write_attribute(&mut self, key: &str, value: &str) -> &mut Self {
        write!(self.doc, " {}=\"{}\"", key, escape(value)).unwrap();
        self
    }

    pub fn write_ns(self, namespace: &str, prefix: Option<&str>) -> Self {
        match prefix {
            Some(prefix) => {
                write!(self.doc, " xmlns:{}=\"{}\"", prefix, escape(namespace)).unwrap()
            }
            None => write!(self.doc, " xmlns=\"{}\"", escape(namespace)).unwrap(),
        }
        self
    }

    pub fn finish(self) -> ScopeWriter<'a, 'b> {
        write!(self.doc, ">").unwrap();
        ScopeWriter {
            doc: self.doc,
            start: self.start,
        }
    }
}

/// Wrap the construction of a tag pair `<a></a>`
pub struct ScopeWriter<'a, 'b> {
    doc: &'a mut String,
    start: &'b str,
}

impl Drop for ScopeWriter<'_, '_> {
    fn drop(&mut self) {
        write!(self.doc, "</{}>", self.start).unwrap();
    }
}

impl ScopeWriter<'_, '_> {
    pub fn data(&mut self, data: &str) {
        self.doc.write_str(escape(data).as_ref()).unwrap();
    }

    pub fn finish(self) {
        // drop will be called which writes the closer to the document
    }

    pub fn start_el<'b, 'c>(&'c mut self, tag: &'b str) -> ElWriter<'c, 'b> {
        write!(self.doc, "<{}", tag).unwrap();
        ElWriter {
            doc: self.doc,
            start: tag,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::encode::XmlWriter;
    use protocol_test_helpers::{assert_ok, validate_body, MediaType};

    #[test]
    fn basic_document_encoding() {
        let mut out = String::new();
        let mut doc_writer = XmlWriter::new(&mut out);
        let mut start_el = doc_writer
            .start_el("Hello")
            .write_ns("http://example.com", None);
        start_el.write_attribute("key", "foo");
        let mut tag = start_el.finish();
        let mut inner = tag.start_el("inner").finish();
        inner.data("hello world!");
        inner.finish();
        let more_inner = tag.start_el("inner").finish();
        more_inner.finish();
        tag.finish();

        assert_ok(validate_body(
            out,
            r#"<Hello key="foo" xmlns="http://example.com">
                    <inner>hello world!</inner>
                    <inner></inner>
                </Hello>"#,
            MediaType::Xml,
        ));
    }

    #[test]
    fn escape_data() {
        let mut s = String::new();
        {
            let mut doc_writer = XmlWriter::new(&mut s);
            let mut start_el = doc_writer.start_el("Hello");
            start_el.write_attribute("key", "<key=\"value\">");
            let mut tag = start_el.finish();
            tag.data("\n\r&");
        }
        assert_eq!(
            s,
            r#"<Hello key="&lt;key=&quot;value&quot;&gt;">&#xA;&#xD;&amp;</Hello>"#
        )
    }
}

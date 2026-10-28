/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_schema::codec::Codec;
use aws_smithy_schema::serde::{SerdeError, ShapeDeserializer};
use aws_smithy_schema::{shape_id, Schema, ShapeType};
use aws_smithy_xml::codec::{XmlCodec, XmlCodecSettings};

static TEXT: Schema<'static> = Schema::new_member(
    shape_id!("compat.audit", "XmlBodyInput", "text"),
    ShapeType::String,
    "text",
    0,
)
.with_xml_name("renamed");
static TOKEN: Schema<'static> = Schema::new_member(
    shape_id!("compat.audit", "XmlBodyInput", "token"),
    ShapeType::String,
    "token",
    1,
)
.with_xml_attribute();
static INPUT: Schema<'static> = Schema::new_struct(
    shape_id!("compat.audit", "XmlBodyInput"),
    ShapeType::Structure,
    &[&TEXT, &TOKEN],
);
static ITEM: Schema<'static> = Schema::new_member(
    shape_id!("test", "List", "member"),
    ShapeType::String,
    "member",
    0,
);
static LIST: Schema<'static> = Schema::new_list(shape_id!("test", "List"), &ITEM);
static KEY: Schema<'static> =
    Schema::new_member(shape_id!("test", "Map", "key"), ShapeType::String, "key", 0);
static VALUE: Schema<'static> = Schema::new_member(
    shape_id!("test", "Map", "value"),
    ShapeType::String,
    "value",
    1,
);
static MAP: Schema<'static> = Schema::new_map(shape_id!("test", "Map"), &KEY, &VALUE);

fn codec(validate: bool) -> XmlCodec {
    XmlCodec::new(
        XmlCodecSettings::builder()
            .validate_document(validate)
            .build(),
    )
}

fn read_input(codec: &XmlCodec, input: &str) -> Result<Vec<String>, SerdeError> {
    let mut values = Vec::new();
    codec
        .create_deserializer(input.as_bytes())
        .read_struct(&INPUT, &mut |member, d| {
            values.push(d.read_string(member)?);
            Ok(())
        })?;
    Ok(values)
}

#[test]
fn document_validation_rejects_duplicate_attributes_and_skipped_references() {
    // Generated legacy restXml serde (schemaSerde=false) accepted these inputs:
    // HTTP 200, content-length: 0, empty body. Only the opt-in validation mode
    // rejects them; default decoding retains the observed legacy behavior.
    for input in [
        r#"<XmlBodyInput token="one" token="two"><renamed>ok</renamed></XmlBodyInput>"#,
        r#"<XmlBodyInput><unknown a="one" a="two"/><renamed>ok</renamed></XmlBodyInput>"#,
        "<XmlBodyInput><unknown>&undefined;</unknown><renamed>ok</renamed></XmlBodyInput>",
        "<XmlBodyInput><unknown>&#0;</unknown><renamed>ok</renamed></XmlBodyInput>",
        "<XmlBodyInput><unknown>raw & text</unknown><renamed>ok</renamed></XmlBodyInput>",
        "<XmlBodyInput><renamed>ok</renamed></XmlBodyInput>junk",
        "<XmlBodyInput><renamed>ok</renamed></Wrong>",
    ] {
        assert!(read_input(&codec(false), input).is_ok(), "{input}");
        assert!(read_input(&codec(true), input).is_err(), "{input}");
    }
}

#[test]
fn document_validation_checks_reference_grammar_and_xml_character_ranges() {
    for entity in [
        "&undefined;",
        "&",
        "&#;",
        "&#x;",
        "&#-1;",
        "&#+65;",
        "&#x+41;",
        "&#0;",
        "&#11;",
        "&#xD800;",
        "&#xFFFE;",
        "&#x110000;",
        "&#99999999999999999999;",
    ] {
        for content in [
            format!("<unknown>{entity}</unknown>"),
            format!(r#"<unknown a="{entity}"/>"#),
            // Draining skipped scopes must also validate nested attributes.
            format!(r#"<unknown><nested a="{entity}"/></unknown>"#),
        ] {
            let input = format!("<XmlBodyInput>{content}<renamed>ok</renamed></XmlBodyInput>");
            assert!(read_input(&codec(true), &input).is_err(), "{input}");
        }
    }
    let input = r#"<XmlBodyInput token="&quot;&apos;&amp;">
        <unknown>&lt;&gt;&#9;&#10;&#13;&#32;&#xD7FF;&#xE000;&#xFFFD;&#x10000;&#x10FFFF;</unknown>
        <renamed>ok</renamed>
        <!-- &undefined; --><![CDATA[&undefined;]]><?pi &undefined;?>
    </XmlBodyInput><!-- trailing -->"#;
    assert_eq!(read_input(&codec(true), input).unwrap(), ["\"'&", "ok"]);
    let input = r#"<XmlBodyInput xmlns:a="urn:a" xmlns:b="urn:b" token="one">
        <unknown token="two" a:value="a" b:value="b"><nested token="three"/></unknown>
        <unknown token="four"/><renamed>ok</renamed>
    </XmlBodyInput>"#;
    assert_eq!(read_input(&codec(true), input).unwrap(), ["one", "ok"]);
}

#[test]
fn document_validation_applies_to_generic_and_specialized_list_reads() {
    for reader in 0..5 {
        let value = if reader == 2 { "MQ==" } else { "1" };
        for ending in ["</List>", "</Wrong>", "</List>junk", ""] {
            let input = format!("<List><member>{value}</member>{ending}");
            for validate in [false, true] {
                let mut d = codec(validate).create_deserializer(input.as_bytes());
                let result = match reader {
                    0 => d.read_list(&LIST, &mut |d| d.read_string(&ITEM).map(|_| ())),
                    1 => d.read_string_list(&LIST).map(|_| ()),
                    2 => d.read_blob_list(&LIST).map(|_| ()),
                    3 => d.read_integer_list(&LIST).map(|_| ()),
                    _ => d.read_long_list(&LIST).map(|_| ()),
                };
                assert_eq!(
                    result.is_ok(),
                    !validate || ending == "</List>",
                    "reader {reader}, validate {validate}, {input}"
                );
            }
        }
    }
}

#[test]
fn document_validation_applies_to_generic_and_specialized_map_reads() {
    for generic in [false, true] {
        for ending in ["</Map>", "</Wrong>", "</Map>junk", ""] {
            let input = format!("<Map><entry><key>k</key><value>v</value></entry>{ending}");
            for validate in [false, true] {
                let mut d = codec(validate).create_deserializer(input.as_bytes());
                let result = if generic {
                    d.read_map(&MAP, &mut |_, d| d.read_string(&VALUE).map(|_| ()))
                } else {
                    d.read_string_string_map(&MAP).map(|_| ())
                };
                assert_eq!(
                    result.is_ok(),
                    !validate || ending == "</Map>",
                    "generic {generic}, validate {validate}, {input}"
                );
            }
        }
    }
}

#[test]
fn document_validation_applies_to_scalar_reads_and_drains_remaining_text() {
    for input in [
        "<value>ok</Wrong>",
        "<value>ok</value>junk",
        "<value>ok",
        "<value>ok<!-- split -->&undefined;</value>",
        "<value>ok<!-- split -->&#0;</value>",
    ] {
        assert_eq!(
            codec(false)
                .create_deserializer(input.as_bytes())
                .read_string(&TEXT)
                .unwrap(),
            "ok",
        );
        assert!(
            codec(true)
                .create_deserializer(input.as_bytes())
                .read_string(&TEXT)
                .is_err(),
            "{input}"
        );
    }
    assert_eq!(
        codec(true)
            .create_deserializer(b"<value>ok</value><!-- trailing -->")
            .read_string(&TEXT)
            .unwrap(),
        "ok",
    );
}

#[test]
fn document_and_root_name_validation_are_independent() {
    let wrong_root = "<Other><renamed>ok</renamed></Other>";
    let malformed = "<XmlBodyInput><renamed>ok</renamed></Wrong>";
    for document in [false, true] {
        for root in [false, true] {
            let codec = XmlCodec::new(
                XmlCodecSettings::builder()
                    .validate_document(document)
                    .validate_root_name(root)
                    .build(),
            );
            assert_eq!(read_input(&codec, wrong_root).is_ok(), !root);
            assert_eq!(read_input(&codec, malformed).is_ok(), !document);
        }
    }
}

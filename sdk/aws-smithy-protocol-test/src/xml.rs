/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::{pretty_comparison, ProtocolTestFailure};
use roxmltree::{Node, NodeType};
use std::fmt::Write;

/// Assert that two XML documents are equivalent
///
/// This will normalize documents and attempts to determine if it is OK to sort members or not by
/// using a heuristic to determine if the tag represents a list (which should not be reordered)
pub(crate) fn try_xml_equivalent(expected: &str, actual: &str) -> Result<(), ProtocolTestFailure> {
    if expected == actual {
        return Ok(());
    }
    let norm_expected =
        normalize_xml(expected).map_err(|e| ProtocolTestFailure::InvalidBodyFormat {
            expected: "expected document to be valid XML".to_string(),
            found: format!("{e}"),
        })?;
    let norm_actual =
        normalize_xml(actual).map_err(|e| ProtocolTestFailure::InvalidBodyFormat {
            expected: "actual document to be valid XML".to_string(),
            found: format!("{e}\n{actual}"),
        })?;
    if norm_expected == norm_actual {
        Ok(())
    } else {
        Err(ProtocolTestFailure::BodyDidNotMatch {
            comparison: pretty_comparison(&norm_expected, &norm_actual),
            hint: "".to_string(),
        })
    }
}

/// Normalizes XML for comparison during Smithy Protocol tests
///
/// This will normalize documents and attempts to determine if it is OK to sort members or not by
/// using a heuristic to determine if the tag represents a list (which should not be reordered)
pub(crate) fn normalize_xml(s: &str) -> Result<String, roxmltree::Error> {
    let rotree = roxmltree::Document::parse(s)?;
    let root = rotree.root().first_child().unwrap();
    Ok(unparse_tag(root, 1))
}

/// Un-parse a "tag" (a subtree) of an XML document
///
/// This function will first convert each of the tag's children into a normalized string
/// then, assuming the node does not represent a list, it will simply lexicographically sort the fully
/// rendered nodes themselves (avoiding the need to sort on keys then values then attributes, etc.).
///
/// This is not a fast algorithm ;-), but the test data it's running on is not large.
fn unparse_tag(tag: Node<'_, '_>, depth: usize) -> String {
    let mut out = String::new();
    out.push_str(&unparse_start_element(tag));
    let mut child_nodes = tag
        .children()
        // flat_map over empty/ignored nodes
        .flat_map(|node| unparse_node(node, depth + 1))
        .collect::<Vec<_>>();
    if !is_list(tag) {
        child_nodes.sort();
    }
    for node in child_nodes {
        out.push('\n');
        for _ in 0..depth {
            out.push_str("  ");
        }
        out.push_str(&node)
    }
    out.push('\n');
    for _ in 0..depth - 1 {
        out.push_str("  ");
    }
    write!(&mut out, "</{}>", tag.tag_name().name()).unwrap();
    out
}

/// Convert a node into text recursively
///
/// If the node is a start element, it will recursively convert all of its children
/// If the node is text, it will return the text, stripped of whitespace
/// If the node is neither, it is ignored
fn unparse_node(n: Node<'_, '_>, depth: usize) -> Option<String> {
    match n.node_type() {
        NodeType::Element => Some(unparse_tag(n, depth)),
        NodeType::Text => {
            let o = n.text().map(|t| t.trim().to_string())?;
            if o.is_empty() {
                None
            } else {
                Some(o)
            }
        }
        _ => None,
    }
}

/// Convert a node back into a string. Attributes are sorted by key, value, and namespace
///
/// Produces output like: `<a key="foo">`
fn unparse_start_element(n: Node<'_, '_>) -> String {
    let mut out = String::new();
    out.push('<');
    out.push_str(n.tag_name().name());
    for ns in n.namespaces() {
        out.push_str(" xmlns");
        if let Some(ns_name) = ns.name() {
            write!(&mut out, ":{ns_name}").unwrap();
        }
        write!(&mut out, "={}", ns.uri()).unwrap();
    }
    let mut attributes: Vec<_> = n.attributes().iter().collect();
    attributes.sort_by_key(|attrib| (attrib.name(), attrib.value(), attrib.namespace()));
    for attribute in attributes {
        write!(&mut out, " ").unwrap();
        if let Some(ns) = attribute.namespace() {
            write!(&mut out, "{ns}:").unwrap();
        }
        write!(&mut out, "{}=\"{}\"", attribute.name(), attribute.value()).unwrap();
    }

    out.push('>');
    out
}

fn is_list(node: Node<'_, '_>) -> bool {
    // a flat list looks like:
    // <Foo>
    //     <flat>example1</flat>
    //     <flat>example2</flat>
    //     <flat>example3</flat>
    // </Foo>

    // a regular list looks like:
    // <values>
    //     <Item>example1</Item>
    //     <Item>example2</Item>
    //     <Item>example3</Item>
    // </values>

    if !node.has_children() {
        return false;
    }

    // in both of these cases, we don't want to reorder because list ordering is actually important
    let all_children_elements =
        non_empty_children(node).all(|child| child.node_type() == NodeType::Element);
    let first_child = non_empty_children(node)
        .next()
        .expect("we know one child exists");
    let all_same_name =
        non_empty_children(node).all(|child| child.tag_name() == first_child.tag_name());
    let all_have_one_child =
        non_empty_children(node).all(|child| non_empty_children(child).count() == 1);
    all_children_elements && all_same_name && all_have_one_child
}

/// Children of `node` that are not whitespace text nodes
fn non_empty_children<'a, 'input: 'a>(
    node: Node<'a, 'input>,
) -> impl Iterator<Item = Node<'a, 'input>> {
    let single_child = node.children().count() == 1;
    node.children()
        .filter(move |c| single_child || !c.is_text() || !c.text().unwrap().trim().is_empty())
}

#[cfg(test)]
mod test {
    use crate::xml::{is_list, normalize_xml, try_xml_equivalent};
    use pretty_assertions::{assert_eq, assert_ne};
    use std::error::Error;

    #[test]
    fn normalize_field_order() -> Result<(), Box<dyn Error>> {
        let d1 = r#"<SimpleScalarPropertiesInputOutput xmlns="https://example.com" test="test" a="a">
        <stringValue>string</stringValue>
            <trueBooleanValue>true</trueBooleanValue>
            <falseBooleanValue>false</falseBooleanValue>
            <Nested xmlns:xsi="https://example.com" xsi:someName="nestedAttrValue"><a></a></Nested>
            </SimpleScalarPropertiesInputOutput>"#;
        let d2 = r#"<SimpleScalarPropertiesInputOutput xmlns="https://example.com" test="test" a="a">
        <stringValue>string</stringValue>
            <falseBooleanValue>false</falseBooleanValue>
            <trueBooleanValue>true</trueBooleanValue>
            <Nested xmlns:xsi="https://example.com" xsi:someName="nestedAttrValue"><a></a></Nested>
            </SimpleScalarPropertiesInputOutput>"#;
        // sanity check ;-)
        assert_ne!(d1, d2);
        assert_eq!(normalize_xml(d1).unwrap(), normalize_xml(d2).unwrap());
        Ok(())
    }

    #[test]
    fn detect_lists() {
        let d1 = r#"<values>
        <Item>example1</Item>
        <Item>example2</Item>
        <Item>example3</Item>
    </values>"#;

        let rotree = roxmltree::Document::parse(d1).unwrap();
        let root = rotree.root().first_child().unwrap();
        assert!(is_list(root));
    }

    #[test]
    fn dont_reorder_lists() {
        let d1 = r#"<Foo>
    <values>
        <Item>example1</Item>
        <Item>example2</Item>
        <Item>example3</Item>
    </values>
</Foo>"#;
        let d2 = r#"<Foo>
    <values>
        <Item>example1</Item>
        <Item>example3</Item>
        <Item>example2</Item>
    </values>
</Foo>"#;
        try_xml_equivalent(d1, d2).expect_err("lists are out of order");
    }

    #[test]
    fn reorder_wrapped_maps() {
        let d1 = r#"<Foo>
            <values>
                <entry>
                    <key>example-key1</key>
                    <value>example1</value>
                </entry>
                <entry>
                    <key>example-key2</key>
                    <value>example2</value>
                </entry>
            </values>
        </Foo>"#;
        let d2 = r#"<Foo>
            <values>
                <entry>
                    <key>example-key2</key>
                    <value>example2</value>
                </entry>
                <entry>
                    <key>example-key1</key>
                    <value>example1</value>
                </entry>
            </values>
        </Foo>"#;
        assert_eq!(normalize_xml(d1).unwrap(), normalize_xml(d2).unwrap());
    }

    #[test]
    fn reorder_flat_maps() {
        let d1 = r#"
        <Bar>
            <flatMap>
                <key>example-key1</key>
                <value>example1</value>
            </flatMap>
            <flatMap>
                <key>example-key2</key>
                <value>example2</value>
            </flatMap>
            <flatMap>
                <key>example-key3</key>
                <value>example3</value>
            </flatMap>
        </Bar>"#;
        let d2 = r#"
        <Bar>
            <flatMap>
                <value>example1</value>
                <key>example-key1</key>
            </flatMap>
            <flatMap>
                <key>example-key3</key>
                <value>example3</value>
            </flatMap>
            <flatMap>
                <value>example2</value>
                <key>example-key2</key>
            </flatMap>
        </Bar>"#;
        try_xml_equivalent(d1, d2).expect("XML is equivalent except for reordering");
    }

    #[test]
    fn normalize_self_closing_elements() {
        try_xml_equivalent("<a/>", "<a></a>").expect("xml was equivalent");
    }

    #[test]
    fn different_attributes_are_different() {
        let d1 = r#"<XmlAttributesInputOutput test="test">
                  <foo>hi</foo>
              </XmlAttributesInputOutput>"#;
        let d2 = r#"<XmlAttributesInputOutput test="other">
                  <foo>hi</foo>
              </XmlAttributesInputOutput>"#;
        try_xml_equivalent(d1, d2).expect_err("differing attributes");
    }

    #[test]
    fn nested_namespaces() {
        let d1 = r#"<root xmlns="https://example.com/foo">
            <Nested xmlns:xsi="https://example2.com" xsi:someName="nestedAttrValue"></Nested>
        </root>"#;
        let d2 = r#"<root xmlns="https://example.com/foo">
            <Nested xmlns:xsi="https://example3.com" xsi:someName="nestedAttrValue"></Nested>
        </root>"#;
        try_xml_equivalent(d1, d2).expect_err("namespaces differ");
    }

    #[test]
    fn namespace_with_prefix() {
        let d1 = r#"<PayloadWithXmlNamespaceAndPrefix xmlns:baz="http://foo.com" />"#;
        let d2 = r#"<PayloadWithXmlNamespaceAndPrefix xmlns:baz="http://foo.com"></PayloadWithXmlNamespaceAndPrefix>"#;
        try_xml_equivalent(d1, d2).expect("match")
    }
}

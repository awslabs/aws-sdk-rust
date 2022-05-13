/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Functions shared between the tests of several modules.

use bytes::Bytes;
use http::{Method, Request, Uri, Version};
use std::error::Error as StdError;

fn path(name: &str, ext: &str) -> String {
    format!("aws-sig-v4-test-suite/{}/{}.{}", name, name, ext)
}

fn read(path: &str) -> String {
    println!("Loading `{}` for test case...", path);
    match std::fs::read_to_string(path) {
        // This replacement is necessary for tests to pass on Windows, as reading the
        // sigv4 snapshots from the file system results in CRLF line endings being inserted.
        Ok(value) => value.replace("\r\n", "\n"),
        Err(err) => {
            panic!("failed to load test case `{}`: {}", path, err);
        }
    }
}

pub(crate) fn test_canonical_request(name: &str) -> String {
    // Tests fail if there's a trailing newline in the file, and pre-commit requires trailing newlines
    read(&path(name, "creq")).trim().to_string()
}

pub(crate) fn test_sts(name: &str) -> String {
    read(&path(name, "sts"))
}

pub(crate) fn test_request(name: &str) -> Request<Bytes> {
    test_parsed_request(name, "req")
}

pub(crate) fn test_signed_request(name: &str) -> Request<Bytes> {
    test_parsed_request(name, "sreq")
}

pub(crate) fn test_signed_request_query_params(name: &str) -> Request<Bytes> {
    test_parsed_request(name, "qpsreq")
}

fn test_parsed_request(name: &str, ext: &str) -> Request<Bytes> {
    let path = path(name, ext);
    match parse_request(read(&path).as_bytes()) {
        Ok(parsed) => parsed,
        Err(err) => panic!("Failed to parse {}: {}", path, err),
    }
}

pub(crate) fn make_headers_comparable<B>(request: &mut Request<B>) {
    for (_name, value) in request.headers_mut() {
        value.set_sensitive(false);
    }
}

fn parse_request(
    s: &[u8],
) -> Result<Request<bytes::Bytes>, Box<dyn StdError + Send + Sync + 'static>> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    // httparse 1.5 requires two trailing newlines to head the header section.
    let mut with_newline = Vec::from(s);
    with_newline.push(b'\n');
    let mut req = httparse::Request::new(&mut headers);
    let _ = req.parse(&with_newline).unwrap();

    let version = match req.version.unwrap() {
        1 => Version::HTTP_11,
        _ => unimplemented!(),
    };

    let method = match req.method.unwrap() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        _ => unimplemented!(),
    };

    let mut builder = Request::builder();
    builder = builder.version(version);
    builder = builder.method(method);

    let mut uri_builder = Uri::builder().scheme("https");
    if let Some(path) = req.path {
        uri_builder = uri_builder.path_and_query(path);
    }
    for header in req.headers {
        let name = header.name.to_lowercase();
        if name == "host" {
            uri_builder = uri_builder.authority(header.value);
        } else if !name.is_empty() {
            builder = builder.header(&name, header.value);
        }
    }

    builder = builder.uri(uri_builder.build()?);
    let req = builder.body(bytes::Bytes::new())?;
    Ok(req)
}

#[test]
fn test_parse_headers() {
    let buf = b"Host:example.amazonaws.com\nX-Amz-Date:20150830T123600Z\n\nblah blah";
    let mut headers = [httparse::EMPTY_HEADER; 4];
    assert_eq!(
        httparse::parse_headers(buf, &mut headers),
        Ok(httparse::Status::Complete((
            56,
            &[
                httparse::Header {
                    name: "Host",
                    value: b"example.amazonaws.com",
                },
                httparse::Header {
                    name: "X-Amz-Date",
                    value: b"20150830T123600Z",
                }
            ][..]
        )))
    );
}

#[test]
fn test_parse() {
    test_request("post-header-key-case");
}

#[test]
fn test_read_query_params() {
    test_request("get-vanilla-query-order-key-case");
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;

use aws_smithy_cbor::decode::Decoder;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn str_benchmark(c: &mut Criterion) {
    // Definite length key `thisIsAKey`.
    let definite_bytes = [
        0x6a, 0x74, 0x68, 0x69, 0x73, 0x49, 0x73, 0x41, 0x4b, 0x65, 0x79,
    ];

    // Indefinite length key `this`, `Is`, `A` and `Key`.
    let indefinite_bytes = [
        0x7f, 0x64, 0x74, 0x68, 0x69, 0x73, 0x62, 0x49, 0x73, 0x61, 0x41, 0x63, 0x4b, 0x65, 0x79,
        0xff,
    ];

    c.bench_function("definite str()", |b| {
        b.iter(|| {
            let mut decoder = Decoder::new(&definite_bytes);
            let x = black_box(decoder.str());
            assert!(matches!(x.unwrap().as_ref(), "thisIsAKey"));
        })
    });

    c.bench_function("definite str_alt", |b| {
        b.iter(|| {
            let mut decoder = minicbor::decode::Decoder::new(&indefinite_bytes);
            let x = black_box(str_alt(&mut decoder));
            assert!(matches!(x.unwrap().as_ref(), "thisIsAKey"));
        })
    });

    c.bench_function("indefinite str()", |b| {
        b.iter(|| {
            let mut decoder = Decoder::new(&indefinite_bytes);
            let x = black_box(decoder.str());
            assert!(matches!(x.unwrap().as_ref(), "thisIsAKey"));
        })
    });

    c.bench_function("indefinite str_alt", |b| {
        b.iter(|| {
            let mut decoder = minicbor::decode::Decoder::new(&indefinite_bytes);
            let x = black_box(str_alt(&mut decoder));
            assert!(matches!(x.unwrap().as_ref(), "thisIsAKey"));
        })
    });
}

// The following seems to be a bit slower than the implementation that we have
// kept in the `aws_smithy_cbor::Decoder`.
pub fn string_alt<'b>(
    decoder: &'b mut minicbor::Decoder<'b>,
) -> Result<String, minicbor::decode::Error> {
    decoder.str_iter()?.collect()
}

// The following seems to be a bit slower than the implementation that we have
// kept in the `aws_smithy_cbor::Decoder`.
fn str_alt<'b>(
    decoder: &'b mut minicbor::Decoder<'b>,
) -> Result<Cow<'b, str>, minicbor::decode::Error> {
    // This implementation uses `next` twice to see if there is
    // another str chunk. If there is, it returns a owned `String`.
    let mut chunks_iter = decoder.str_iter()?;
    let head = match chunks_iter.next() {
        Some(Ok(head)) => head,
        None => return Ok(Cow::Borrowed("")),
        Some(Err(e)) => return Err(e),
    };

    match chunks_iter.next() {
        None => Ok(Cow::Borrowed(head)),
        Some(Err(e)) => Err(e),
        Some(Ok(next)) => {
            let mut concatenated_string = String::from(head);
            concatenated_string.push_str(next);
            for chunk in chunks_iter {
                concatenated_string.push_str(chunk?);
            }
            Ok(Cow::Owned(concatenated_string))
        }
    }
}

// We have two `string` implementations. One uses `collect` the other
// uses `String::new` followed by `string::push`.
pub fn string_benchmark(c: &mut Criterion) {
    // Definite length key `thisIsAKey`.
    let definite_bytes = [
        0x6a, 0x74, 0x68, 0x69, 0x73, 0x49, 0x73, 0x41, 0x4b, 0x65, 0x79,
    ];

    // Indefinite length key `this`, `Is`, `A` and `Key`.
    let indefinite_bytes = [
        0x7f, 0x64, 0x74, 0x68, 0x69, 0x73, 0x62, 0x49, 0x73, 0x61, 0x41, 0x63, 0x4b, 0x65, 0x79,
        0xff,
    ];

    c.bench_function("definite string()", |b| {
        b.iter(|| {
            let mut decoder = Decoder::new(&definite_bytes);
            let _ = black_box(decoder.string());
        })
    });

    c.bench_function("definite string_alt()", |b| {
        b.iter(|| {
            let mut decoder = minicbor::decode::Decoder::new(&indefinite_bytes);
            let _ = black_box(string_alt(&mut decoder));
        })
    });

    c.bench_function("indefinite string()", |b| {
        b.iter(|| {
            let mut decoder = Decoder::new(&indefinite_bytes);
            let _ = black_box(decoder.string());
        })
    });

    c.bench_function("indefinite string_alt()", |b| {
        b.iter(|| {
            let mut decoder = minicbor::decode::Decoder::new(&indefinite_bytes);
            let _ = black_box(string_alt(&mut decoder));
        })
    });
}

criterion_group!(benches, string_benchmark, str_benchmark,);
criterion_main!(benches);

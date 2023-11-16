/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![no_main]

use aws_smithy_eventstream::frame::{read_message_from, write_message_to};
use aws_smithy_types::event_stream::{Header, HeaderValue, Message};
use aws_smithy_types::DateTime;
use bytes::{Buf, BufMut};
use crc32fast::Hasher as Crc;
use libfuzzer_sys::{fuzz_mutator, fuzz_target};

// This fuzz test uses a custom mutator to manipulate the headers.
// If it fails to parse a message from the unmutated input, it will create a message
// with every single possible header type to give the fuzzer a leg up.
// After the headers are mutated, a new valid prelude and valid message CRC are generated
// so that the fuzzer can actually explore the header parsing logic.
fn mutate(data: &mut [u8], size: usize, max_size: usize) -> usize {
    let input = &mut &data[..size];
    let message = if let Ok(message) = read_message_from(input) {
        message
    } else {
        Message::new(&b"some payload"[..])
            .add_header(Header::new("true", HeaderValue::Bool(true)))
            .add_header(Header::new("false", HeaderValue::Bool(false)))
            .add_header(Header::new("byte", HeaderValue::Byte(50)))
            .add_header(Header::new("short", HeaderValue::Int16(20_000)))
            .add_header(Header::new("int", HeaderValue::Int32(500_000)))
            .add_header(Header::new("long", HeaderValue::Int64(50_000_000_000)))
            .add_header(Header::new(
                "bytes",
                HeaderValue::ByteArray((&b"some bytes"[..]).into()),
            ))
            .add_header(Header::new("str", HeaderValue::String("some str".into())))
            .add_header(Header::new(
                "time",
                HeaderValue::Timestamp(DateTime::from_secs(5_000_000_000)),
            ))
            .add_header(Header::new(
                "uuid",
                HeaderValue::Uuid(0xb79bc914_de21_4e13_b8b2_bc47e85b7f0b),
            ))
    };

    let mut bytes = Vec::new();
    write_message_to(&message, &mut bytes).unwrap();

    let headers_len = (&bytes[4..8]).get_u32();
    let non_header_len = bytes.len() - headers_len as usize;
    let max_header_len = max_size - non_header_len;
    let mut headers = (&bytes[12..(12 + headers_len as usize)]).to_vec();
    headers.resize(max_header_len, 0);
    let new_header_len =
        libfuzzer_sys::fuzzer_mutate(&mut headers, headers_len as usize, max_header_len);

    let mut mutated = Vec::<u8>::new();
    mutated.put_u32((new_header_len + non_header_len) as u32);
    mutated.put_u32(new_header_len as u32);
    mutated.put_u32(crc(&mutated));
    mutated.put_slice(&headers[..new_header_len]);
    mutated.put_slice(message.payload());
    mutated.put_u32(crc(&mutated));

    data[..mutated.len()].copy_from_slice(&mutated);
    mutated.len()
}

fuzz_mutator!(
    |data: &mut [u8], size: usize, max_size: usize, _seed: u32| { mutate(data, size, max_size) }
);

fuzz_target!(|data: &[u8]| {
    let mut message = data;
    let _ = read_message_from(&mut message);
});

fn crc(input: &[u8]) -> u32 {
    let mut crc = Crc::new();
    crc.update(input);
    crc.finalize()
}

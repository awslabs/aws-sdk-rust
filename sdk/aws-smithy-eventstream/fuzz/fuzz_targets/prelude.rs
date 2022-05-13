/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![no_main]

use arbitrary::Arbitrary;
use aws_smithy_eventstream::frame::{Header, HeaderValue, Message};
use bytes::{Buf, BufMut};
use crc32fast::Hasher as Crc;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    total_len: u32,
    header_len: u32,
}

// This fuzz test exclusively fuzzes the message prelude while keeping the CRCs valid.
fuzz_target!(|input: Input| {
    let message = Message::new(&b"some payload"[..])
        .add_header(Header::new("str", HeaderValue::String("some str".into())));

    let mut bytes = Vec::new();
    message.write_to(&mut bytes).unwrap();

    let headers_len = (&bytes[4..8]).get_u32();
    let headers = &bytes[12..(12 + headers_len as usize)];

    let mut mutated = Vec::<u8>::new();
    mutated.put_u32(input.total_len);
    mutated.put_u32(input.header_len);
    mutated.put_u32(crc(&mutated));
    mutated.put_slice(headers);
    mutated.put_slice(message.payload());
    mutated.put_u32(crc(&mutated));

    let _ = Message::read_from(&mut &mutated[..]);
});

fn crc(input: &[u8]) -> u32 {
    let mut crc = Crc::new();
    crc.update(input);
    crc.finalize()
}

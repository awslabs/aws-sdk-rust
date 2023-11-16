/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![no_main]

use aws_smithy_eventstream::frame::read_message_from;
use bytes::BufMut;
use crc32fast::Hasher as Crc;
use libfuzzer_sys::fuzz_target;

#[derive(derive_arbitrary::Arbitrary, Debug)]
struct Input {
    headers: Vec<u8>,
    payload: Vec<u8>,
}

// This fuzz test assists the fuzzer by creating a well formed prelude and message CRC
fuzz_target!(|input: Input| {
    let total_len = (12 + input.headers.len() + input.payload.len() + 4) as u32;
    let header_len = input.headers.len() as u32;

    let mut message = Vec::<u8>::new();
    message.put_u32(total_len);
    message.put_u32(header_len);
    message.put_u32(crc(&message));
    message.put_slice(&input.headers);
    message.put_slice(&input.payload);
    message.put_u32(crc(&message));

    let mut data = &mut &message[..];
    let _ = read_message_from(&mut data);
});

fn crc(input: &[u8]) -> u32 {
    let mut crc = Crc::new();
    crc.update(input);
    crc.finalize()
}

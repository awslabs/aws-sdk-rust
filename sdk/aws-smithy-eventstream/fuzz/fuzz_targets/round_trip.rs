/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![no_main]
use aws_smithy_eventstream::arbitrary::ArbMessage;
use aws_smithy_eventstream::frame::{read_message_from, write_message_to};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|message: ArbMessage| {
    let message = message.into();
    let mut buffer = Vec::new();
    match write_message_to(&message, &mut buffer) {
        Ok(_) => {
            let mut data = &buffer[..];
            let parsed = read_message_from(&mut data).unwrap();
            assert_eq!(message, parsed);
        }
        Err(err) => {
            if !err.is_invalid_message() {
                panic!("unexpected error on write: {}", err),
            }
        }
    }
});

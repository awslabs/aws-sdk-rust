/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![no_main]

use aws_smithy_eventstream::frame::read_message_from;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut message = data;
    let _ = read_message_from(&mut message);
});

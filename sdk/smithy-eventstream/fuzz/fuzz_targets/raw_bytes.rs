/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![no_main]

use libfuzzer_sys::fuzz_target;
use smithy_eventstream::frame::Message;

fuzz_target!(|data: &[u8]| {
    let mut message = data;
    let _ = Message::read_from(&mut message);
});

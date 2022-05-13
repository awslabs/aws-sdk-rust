/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![no_main]

use aws_smithy_types::date_time::{DateTime, Format};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = std::str::from_utf8(data) {
        // Looking for panics. Don't care if the parsing fails.
        let _ = DateTime::from_str(value, Format::HttpDate);
    }
});

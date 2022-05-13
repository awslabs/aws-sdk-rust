/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![no_main]
use libfuzzer_sys::fuzz_target;

use aws_smithy_http::header::read_many_from_str;
use http;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(req) = http::Request::builder().header("test", s).body(()) {
            // Shouldn't panic
            let _ = read_many_from_str::<String>(req.headers().get_all("test").iter());
        }
    }
});

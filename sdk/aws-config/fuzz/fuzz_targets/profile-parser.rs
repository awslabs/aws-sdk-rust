/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![no_main]
use aws_config::profile;
use aws_types::os_shim_internal::{Env, Fs};
use libfuzzer_sys::fuzz_target;
use std::collections::HashMap;
use std::ffi::OsString;

// Fuzz on a tuple of (`config`, `credentials`)
fuzz_target!(|data: (Option<&str>, Option<&str>)| {
    let mut fs: HashMap<OsString, Vec<u8>> = HashMap::new();
    if let Some(config) = data.0 {
        fs.insert(
            "~/.aws/config".to_string().into(),
            config.to_string().into(),
        );
    }
    if let Some(creds) = data.1 {
        fs.insert(
            "~/.aws/credentials".to_string().into(),
            creds.to_string().into(),
        );
    }
    let fs = Fs::from_raw_map(fs);
    let env = Env::real();
    let _ = profile::load(&fs, &env);
});

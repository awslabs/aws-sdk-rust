/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

include!(concat!(env!("OUT_DIR"), "/build_env.rs"));

pub struct BuildMetadata {
    pub rust_version: &'static str,
    pub core_pkg_version: &'static str,
    pub os_family: OsFamily,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum OsFamily {
    Windows,
    Linux,
    Macos,
    Android,
    Ios,
    Other,
}

/// Check the current OS against a list of options
///
/// You can't directly inspect the value of a `cfg` param. Instead, you need to check if the param
/// is set to a specific value. This macro simplifies checking the current OS family.
///
/// Usage:
/// ```rust
/// let os = get_os_family!(target_os: ("linux", OsFamily::Windows), ("android", OsFamily::Android));
/// ```
macro_rules! get_os_family {
    ($i:ident : $(($s:expr, $t:expr)),+) => (
        { const fn __getcfg() -> OsFamily { $( if cfg!($i=$s) { return $t; } );+ OsFamily::Other } __getcfg()  }
    )
}

impl OsFamily {
    pub const fn from_env() -> Self {
        // values from https://doc.rust-lang.org/reference/conditional-compilation.html#target_os
        get_os_family!(target_os:
            ("windows", OsFamily::Windows),
            ("macos", OsFamily::Macos),
            ("ios", OsFamily::Ios),
            ("linux", OsFamily::Linux),
            ("android", OsFamily::Android)
        )
    }
}

impl From<&str> for OsFamily {
    fn from(s: &str) -> Self {
        match s {
            "windows" => OsFamily::Windows,
            "macos" => OsFamily::Macos,
            "ios" => OsFamily::Ios,
            "linux" => OsFamily::Linux,
            "android" => OsFamily::Android,
            _ => OsFamily::Other,
        }
    }
}

pub const BUILD_METADATA: BuildMetadata = BuildMetadata {
    rust_version: RUST_VERSION,
    core_pkg_version: env!("CARGO_PKG_VERSION"),
    os_family: OsFamily::from_env(),
};

#[cfg(test)]
mod test {
    use crate::build_metadata::{OsFamily, BUILD_METADATA};

    #[test]
    fn valid_build_metadata() {
        let meta = &BUILD_METADATA;
        // obviously a slightly brittle test. Will be a small update for Rust 2.0 and GA :-)
        assert!(meta.rust_version.starts_with("1."));
        assert!(meta.core_pkg_version.starts_with("0."));
        // quick sanity check that we're parsing common platforms correctly
        if cfg!(target_os = "linux") {
            assert_eq!(meta.os_family, OsFamily::Linux);
        }
        if cfg!(target_os = "macos") {
            assert_eq!(meta.os_family, OsFamily::Macos);
        }
    }
}

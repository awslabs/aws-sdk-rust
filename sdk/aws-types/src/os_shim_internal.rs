/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Abstractions for testing code that interacts with the operating system:
//! - Reading environment variables
//! - Reading from the file system

use std::collections::HashMap;
use std::env::VarError;
use std::ffi::OsString;
use std::path::Path;
use std::sync::Arc;

/// File system abstraction
///
/// Simple abstraction enabling in-memory mocking of the file system
///
/// # Example
/// Construct a file system which delegates to `std::fs`:
/// ```rust
/// let fs = aws_types::os_shim_internal::Fs::real();
/// ```
///
/// Construct an in-memory file system for testing:
/// ```rust
/// use std::collections::HashMap;
/// let fs = aws_types::os_shim_internal::Fs::from_map({
///     let mut map = HashMap::new();
///     map.insert("/home/.aws/config".to_string(), "[default]\nregion = us-east-1".into());
///     map
/// });
/// ```
pub struct Fs(fs::Inner);

impl Fs {
    pub fn real() -> Self {
        Fs(fs::Inner::Real)
    }

    pub fn from_raw_map(fs: HashMap<OsString, Vec<u8>>) -> Self {
        Fs(fs::Inner::Fake { fs })
    }

    pub fn from_map(data: HashMap<String, Vec<u8>>) -> Self {
        let fs = data.into_iter().map(|(k, v)| (k.into(), v)).collect();
        Fs(fs::Inner::Fake { fs })
    }

    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        use fs::Inner;
        let path = path.as_ref();
        match &self.0 {
            Inner::Real => path.exists(),
            Inner::Fake { fs, .. } => fs.contains_key(path.as_os_str()),
        }
    }

    pub fn read_to_end(&self, path: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
        use fs::Inner;
        let path = path.as_ref();
        match &self.0 {
            Inner::Real => std::fs::read(path),
            Inner::Fake { fs } => fs
                .get(path.as_os_str())
                .cloned()
                .ok_or_else(|| std::io::ErrorKind::NotFound.into()),
        }
    }
}

mod fs {
    use std::collections::HashMap;
    use std::ffi::OsString;

    pub enum Inner {
        Real,
        Fake { fs: HashMap<OsString, Vec<u8>> },
    }
}

/// Environment variable abstraction
///
/// Environment variables are global to a process, and, as such, are difficult to test with a multi-
/// threaded test runner like Rust's. This enables loading environment variables either from the
/// actual process environment ([`std::env::var`](std::env::var)) or from a hash map.
///
/// Process environments are cheap to clone:
/// - Faked process environments are wrapped in an internal Arc
/// - Real process environments are pointer-sized
#[derive(Clone)]
pub struct Env(env::Inner);

impl Env {
    pub fn get(&self, k: &str) -> Result<String, VarError> {
        use env::Inner;
        match &self.0 {
            Inner::Real => std::env::var(k),
            Inner::Fake(map) => map.get(k).cloned().ok_or(VarError::NotPresent),
        }
    }

    /// Create a fake process environment from a slice of tuples.
    ///
    /// # Example
    /// ```rust
    /// use aws_types::os_shim_internal::Env;
    /// let mock_env = Env::from_slice(&[
    ///     ("HOME", "/home/myname"),
    ///     ("AWS_REGION", "us-west-2")
    /// ]);
    /// assert_eq!(mock_env.get("HOME").unwrap(), "/home/myname");
    /// ```
    pub fn from_slice<'a>(vars: &[(&'a str, &'a str)]) -> Self {
        use env::Inner;
        Self(Inner::Fake(Arc::new(
            vars.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        )))
    }

    /// Create a process environment that uses the real process environment
    ///
    /// Calls will be delegated to [`std::env::var`](std::env::var).
    pub fn real() -> Self {
        Self(env::Inner::Real)
    }
}

impl From<HashMap<String, String>> for Env {
    fn from(hash_map: HashMap<String, String>) -> Self {
        Self(env::Inner::Fake(Arc::new(hash_map)))
    }
}

mod env {
    use std::collections::HashMap;
    use std::sync::Arc;

    #[derive(Clone)]
    pub enum Inner {
        Real,
        Fake(Arc<HashMap<String, String>>),
    }
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Abstractions for testing code that interacts with the operating system:
//! - Reading environment variables
//! - Reading from the file system

use crate::os_shim_internal::fs::Fake;
use std::collections::HashMap;
use std::env::VarError;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// File system abstraction
///
/// Simple abstraction enabling in-memory mocking of the file system
///
/// # Examples
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
#[derive(Clone, Debug)]
pub struct Fs(fs::Inner);

impl Default for Fs {
    fn default() -> Self {
        Fs::real()
    }
}

impl Fs {
    pub fn real() -> Self {
        Fs(fs::Inner::Real)
    }

    pub fn from_raw_map(fs: HashMap<OsString, Vec<u8>>) -> Self {
        Fs(fs::Inner::Fake(Arc::new(Fake::MapFs(fs))))
    }

    pub fn from_map(data: HashMap<String, Vec<u8>>) -> Self {
        let fs = data.into_iter().map(|(k, v)| (k.into(), v)).collect();
        Self::from_raw_map(fs)
    }

    /// Create a test filesystem rooted in real files
    ///
    /// Creates a test filesystem from the contents of `test_directory` rooted into `namespaced_to`.
    ///
    /// Example:
    /// Given:
    /// ```bash
    /// $ ls
    /// ./my-test-dir/aws-config
    /// ./my-test-dir/aws-config/config
    /// $ cat ./my-test-dir/aws-config/config
    /// test-config
    /// ```
    /// ```rust,no_run
    /// # async fn docs() {
    /// use aws_types::os_shim_internal::{Env, Fs};
    /// let env = Env::from_slice(&[("HOME", "/Users/me")]);
    /// let fs = Fs::from_test_dir("my-test-dir/aws-config", "/Users/me/.aws/config");
    /// assert_eq!(fs.read_to_end("/Users/me/.aws/config").await.unwrap(), b"test-config");
    /// # }
    pub fn from_test_dir(
        test_directory: impl Into<PathBuf>,
        namespaced_to: impl Into<PathBuf>,
    ) -> Self {
        Self(fs::Inner::Fake(Arc::new(Fake::NamespacedFs {
            real_path: test_directory.into(),
            namespaced_to: namespaced_to.into(),
        })))
    }

    /// Read the entire contents of a file
    ///
    /// **Note**: This function is currently `async` primarily for forward compatibility. Currently,
    /// this function does not use Tokio (or any other runtime) to perform IO, the IO is performed
    /// directly within the function.
    pub async fn read_to_end(&self, path: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
        use fs::Inner;
        let path = path.as_ref();
        match &self.0 {
            Inner::Real => std::fs::read(path),
            Inner::Fake(fake) => match fake.as_ref() {
                Fake::MapFs(fs) => fs
                    .get(path.as_os_str())
                    .cloned()
                    .ok_or_else(|| std::io::ErrorKind::NotFound.into()),
                Fake::NamespacedFs {
                    real_path,
                    namespaced_to,
                } => {
                    let actual_path = path
                        .strip_prefix(namespaced_to)
                        .map_err(|_| std::io::Error::from(std::io::ErrorKind::NotFound))?;
                    std::fs::read(real_path.join(actual_path))
                }
            },
        }
    }
}

mod fs {
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    pub enum Inner {
        Real,
        Fake(Arc<Fake>),
    }

    #[derive(Debug)]
    pub enum Fake {
        MapFs(HashMap<OsString, Vec<u8>>),
        NamespacedFs {
            real_path: PathBuf,
            namespaced_to: PathBuf,
        },
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
#[derive(Clone, Debug)]
pub struct Env(env::Inner);

impl Default for Env {
    fn default() -> Self {
        Self::real()
    }
}

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
    /// # Examples
    /// ```rust
    /// use aws_types::os_shim_internal::Env;
    /// let mock_env = Env::from_slice(&[
    ///     ("HOME", "/home/myname"),
    ///     ("AWS_REGION", "us-west-2")
    /// ]);
    /// assert_eq!(mock_env.get("HOME").unwrap(), "/home/myname");
    /// ```
    pub fn from_slice<'a>(vars: &[(&'a str, &'a str)]) -> Self {
        let map: HashMap<_, _> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Self::from(map)
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

    #[derive(Clone, Debug)]
    pub enum Inner {
        Real,
        Fake(Arc<HashMap<String, String>>),
    }
}

#[cfg(test)]
mod test {
    use crate::os_shim_internal::{Env, Fs};
    use futures_util::FutureExt;
    use std::env::VarError;

    #[test]
    fn env_works() {
        let env = Env::from_slice(&[("FOO", "BAR")]);
        assert_eq!(env.get("FOO").unwrap(), "BAR");
        assert_eq!(
            env.get("OTHER").expect_err("no present"),
            VarError::NotPresent
        )
    }

    #[test]
    fn fs_works() {
        let fs = Fs::from_test_dir(".", "/users/test-data");
        let _ = fs
            .read_to_end("/users/test-data/Cargo.toml")
            .now_or_never()
            .expect("future should not poll")
            .expect("file exists");

        let _ = fs
            .read_to_end("doesntexist")
            .now_or_never()
            .expect("future should not poll")
            .expect_err("file doesnt exists");
    }
}

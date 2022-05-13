/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Abstractions for testing code that interacts with the operating system:
//! - Reading environment variables
//! - Reading from the file system

use std::collections::HashMap;
use std::env::VarError;
use std::ffi::OsString;
use std::fmt::Debug;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::os_shim_internal::fs::Fake;
use crate::os_shim_internal::time_source::Inner;

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
///     map.insert("/home/.aws/config".to_string(), "[default]\nregion = us-east-1");
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

    pub fn from_map(data: HashMap<String, impl Into<Vec<u8>>>) -> Self {
        let fs = data
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
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

    /// Create a fake process environment from a slice of tuples.
    ///
    /// # Examples
    /// ```rust
    /// # async fn example() {
    /// use aws_types::os_shim_internal::Fs;
    /// let mock_fs = Fs::from_slice(&[
    ///     ("config", "[default]\nretry_mode = \"standard\""),
    /// ]);
    /// assert_eq!(mock_fs.read_to_end("config").await.unwrap(), b"[default]\nretry_mode = \"standard\"");
    /// # }
    /// ```
    pub fn from_slice<'a>(files: &[(&'a str, &'a str)]) -> Self {
        let fs: HashMap<String, Vec<u8>> = files
            .iter()
            .map(|(k, v)| {
                let k = (*k).to_owned();
                let v = v.as_bytes().to_vec();
                (k, v)
            })
            .collect();

        Self::from_map(fs)
    }

    /// Read the entire contents of a file
    ///
    /// _Note: This function is currently `async` primarily for forward compatibility. Currently,
    /// this function does not use Tokio (or any other runtime) to perform IO, the IO is performed
    /// directly within the function._
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
    pub(super) enum Inner {
        Real,
        Fake(Arc<Fake>),
    }

    #[derive(Debug)]
    pub(super) enum Fake {
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
    pub(super) enum Inner {
        Real,
        Fake(Arc<HashMap<String, String>>),
    }
}

#[derive(Debug, Clone)]
pub struct TimeSource(time_source::Inner);

impl TimeSource {
    pub fn real() -> Self {
        TimeSource(time_source::Inner::Real)
    }

    pub fn manual(time_source: &ManualTimeSource) -> Self {
        TimeSource(time_source::Inner::Manual(time_source.clone()))
    }

    pub fn now(&self) -> SystemTime {
        match &self.0 {
            Inner::Real => SystemTime::now(),
            Inner::Manual(manual) => manual.now(),
        }
    }
}

impl Default for TimeSource {
    fn default() -> Self {
        TimeSource::real()
    }
}

/// Time Source that can be manually moved for tests
///
/// # Examples
///
/// ```rust
/// # struct Client {
/// #  // stub
/// # }
/// #
/// # impl Client {
/// #     fn with_timesource(ts: TimeSource) -> Self {
/// #         Client { }
/// #     }
/// # }
/// use aws_types::os_shim_internal::{ManualTimeSource, TimeSource};
/// use std::time::{UNIX_EPOCH, Duration};
/// let mut time = ManualTimeSource::new(UNIX_EPOCH);
/// let client = Client::with_timesource(TimeSource::manual(&time));
/// time.advance(Duration::from_secs(100));
/// ```
#[derive(Clone, Debug)]
pub struct ManualTimeSource {
    queries: Arc<Mutex<Vec<SystemTime>>>,
    now: Arc<Mutex<SystemTime>>,
}

impl ManualTimeSource {
    pub fn new(start_time: SystemTime) -> Self {
        Self {
            queries: Default::default(),
            now: Arc::new(Mutex::new(start_time)),
        }
    }

    pub fn set_time(&mut self, time: SystemTime) {
        let mut now = self.now.lock().unwrap();
        *now = time;
    }

    pub fn advance(&mut self, delta: Duration) {
        let mut now = self.now.lock().unwrap();
        *now += delta;
    }

    pub fn queries(&self) -> impl Deref<Target = Vec<SystemTime>> + '_ {
        self.queries.lock().unwrap()
    }

    pub fn now(&self) -> SystemTime {
        let ts = *self.now.lock().unwrap();
        self.queries.lock().unwrap().push(ts);
        ts
    }
}

mod time_source {
    use crate::os_shim_internal::ManualTimeSource;

    // in the future, if needed we can add a time source trait, however, the manual time source
    // should cover most test use cases.
    #[derive(Debug, Clone)]
    pub(super) enum Inner {
        Real,
        Manual(ManualTimeSource),
    }
}

#[cfg(test)]
mod test {
    use std::env::VarError;
    use std::time::{Duration, UNIX_EPOCH};

    use futures_util::FutureExt;

    use crate::os_shim_internal::{Env, Fs, ManualTimeSource, TimeSource};

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

    #[test]
    fn ts_works() {
        let real = TimeSource::real();
        // no panics
        let _ = real.now();

        let mut manual = ManualTimeSource::new(UNIX_EPOCH);
        let ts = TimeSource::manual(&manual);
        assert_eq!(ts.now(), UNIX_EPOCH);
        manual.advance(Duration::from_secs(10));
        assert_eq!(ts.now(), UNIX_EPOCH + Duration::from_secs(10));
    }
}

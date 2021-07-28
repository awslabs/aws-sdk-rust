/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::os_shim_internal;
use std::borrow::Cow;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

/// In-memory source of profile data
pub struct Source {
    /// Contents and path of ~/.aws/config
    pub config_file: File,

    /// Contents and path of ~/.aws/credentials
    pub credentials_file: File,

    /// Profile to use
    ///
    /// Overridden via `$AWS_PROFILE`, defaults to `default`
    pub profile: Cow<'static, str>,
}

/// In-memory configuration file
pub struct File {
    pub path: String,
    pub contents: String,
}

#[derive(Clone, Copy)]
pub enum FileKind {
    Config,
    Credentials,
}

impl FileKind {
    fn default_path(&self) -> &'static str {
        match &self {
            FileKind::Credentials => "~/.aws/credentials",
            FileKind::Config => "~/.aws/config",
        }
    }

    fn override_environment_variable(&self) -> &'static str {
        match &self {
            FileKind::Config => "AWS_CONFIG_FILE",
            FileKind::Credentials => "AWS_SHARED_CREDENTIALS_FILE",
        }
    }
}

/// Load a [Source](Source) from a given environment and filesystem.
pub fn load(proc_env: &os_shim_internal::Env, fs: &os_shim_internal::Fs) -> Source {
    let home = home_dir(&proc_env, Os::real());
    let config = tracing::info_span!("load_config_file")
        .in_scope(|| load_config_file(FileKind::Config, &home, &fs, &proc_env));
    let credentials = tracing::info_span!("load_credentials_file")
        .in_scope(|| load_config_file(FileKind::Credentials, &home, &fs, &proc_env));
    Source {
        config_file: config,
        credentials_file: credentials,
        profile: proc_env
            .get("AWS_PROFILE")
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed("default")),
    }
}

/// Loads an AWS Config file
///
/// Both the default & the overriding patterns may contain `~/` which MUST be expanded to the users
/// home directory in a platform-aware way (see [`expand_home`](expand_home))
///
/// Arguments:
/// * `kind`: The type of config file to load
/// * `home_directory`: Home directory to use during home directory expansion
/// * `fs`: Filesystem abstraction
/// * `environment`: Process environment abstraction
fn load_config_file(
    kind: FileKind,
    home_directory: &Option<String>,
    fs: &os_shim_internal::Fs,
    environment: &os_shim_internal::Env,
) -> File {
    let path = environment
        .get(kind.override_environment_variable())
        .map(Cow::Owned)
        .ok()
        .unwrap_or_else(|| kind.default_path().into());
    let expanded = expand_home(path.as_ref(), home_directory);
    if path != expanded.to_string_lossy() {
        tracing::debug!(before = ?path, after = ?expanded, "home directory expanded");
    }
    // read the data at the specified path
    // if the path does not exist, log a warning but pretend it was actually an empty file
    let data = match fs.read_to_end(&expanded) {
        Ok(data) => data,
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound if path == kind.default_path() => {
                    tracing::info!(path = %path, "config file not found")
                }
                ErrorKind::NotFound if path != kind.default_path() => {
                    // in the case where the user overrode the path with an environment variable,
                    // log more loudly than the case where the default path was missing
                    tracing::warn!(path = %path, env = %kind.override_environment_variable(), "config file overridden via environment variable not found")
                }
                _other => tracing::warn!(path = %path, error = %e, "failed to read config file"),
            };
            Default::default()
        }
    };
    // if the file is not valid utf-8, log a warning and use an empty file instead
    let data = match String::from_utf8(data) {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!(path = %path, error = %e, "config file did not contain utf-8 encoded data");
            Default::default()
        }
    };
    tracing::info!(path = %path, size = ?data.len(), "config file loaded");
    File {
        // lossy is OK here, the name of this file is just for debugging purposes
        path: expanded.to_string_lossy().into(),
        contents: data,
    }
}

fn expand_home(path: impl AsRef<Path>, home_dir: &Option<String>) -> PathBuf {
    let path = path.as_ref();
    let mut components = path.components();
    let start = components.next();
    match start {
        None => path.into(), // empty path,
        Some(Component::Normal(s)) if s == "~" => {
            // do homedir replacement
            let path = match home_dir {
                Some(dir) => {
                    tracing::debug!(home = ?dir, path = ?path, "performing home directory substitution");
                    dir.clone()
                }
                None => {
                    tracing::warn!(
                        "could not determine home directory but home expansion was requested"
                    );
                    // if we can't determine the home directory, just leave it as `~`
                    "~".into()
                }
            };
            let mut path: PathBuf = path.into();
            // rewrite the path using system-specific path separators
            for component in components {
                path.push(component);
            }
            path
        }
        // Finally, handle the case where it doesn't begin with some version of `~/`:
        // NOTE: in this case we aren't performing path rewriting. This is correct because
        // this path comes from an environment variable on the target
        // platform, so in that case, the separators should already be correct.
        _other => path.into(),
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Os {
    Windows,
    NotWindows,
}

impl Os {
    pub fn real() -> Self {
        match std::env::consts::OS {
            "windows" => Os::Windows,
            _ => Os::NotWindows,
        }
    }
}

/// Resolve a home directory given a set of environment variables
fn home_dir(env_var: &os_shim_internal::Env, os: Os) -> Option<String> {
    if let Ok(home) = env_var.get("HOME") {
        tracing::debug!(src = "HOME", "loaded home directory");
        return Some(home);
    }

    if os == Os::Windows {
        if let Ok(home) = env_var.get("USERPROFILE") {
            tracing::debug!(src = "USERPROFILE", "loaded home directory");
            return Some(home);
        }

        let home_drive = env_var.get("HOMEDRIVE");
        let home_path = env_var.get("HOMEPATH");
        tracing::debug!(src = "HOMEDRIVE/HOMEPATH", "loaded home directory");
        if let (Ok(mut drive), Ok(path)) = (home_drive, home_path) {
            drive.push_str(&path);
            return Some(drive);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::os_shim_internal::{Env, Fs};
    use crate::profile::source::{expand_home, home_dir, load, Os};
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fs;

    #[test]
    fn only_expand_home_prefix() {
        // ~ is only expanded as a single component (currently)
        let path = "~aws/config";
        assert_eq!(expand_home(&path, &None).to_str().unwrap(), "~aws/config");
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct SourceTests {
        tests: Vec<TestCase>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct TestCase {
        name: String,
        environment: HashMap<String, String>,
        platform: String,
        profile: Option<String>,
        config_location: String,
        credentials_location: String,
    }

    /// Run all tests from file-location-tests.json
    #[test]
    fn run_tests() -> Result<(), Box<dyn Error>> {
        let tests = fs::read_to_string("test-data/file-location-tests.json")?;
        let tests: SourceTests = serde_json::from_str(&tests)?;
        for (i, test) in tests.tests.into_iter().enumerate() {
            eprintln!("test: {}", i);
            check(test);
        }
        Ok(())
    }

    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn logs_produced_default() {
        let env = Env::from_slice(&[("HOME", "/user/name")]);
        let mut fs = HashMap::new();
        fs.insert(
            "/user/name/.aws/config".to_string(),
            "[default]\nregion = us-east-1".into(),
        );

        let fs = Fs::from_map(fs);

        let _src = load(&env, &fs);
        assert!(logs_contain("config file loaded"));
        assert!(logs_contain("performing home directory substitution"));
    }

    fn check(test_case: TestCase) {
        let fs = Fs::real();
        let env = Env::from(test_case.environment);
        let platform_matches = (cfg!(windows) && test_case.platform == "windows")
            || (!cfg!(windows) && test_case.platform != "windows");
        if platform_matches {
            let source = load(&env, &fs);
            if let Some(expected_profile) = test_case.profile {
                assert_eq!(source.profile, expected_profile, "{}", &test_case.name);
            }
            assert_eq!(
                source.config_file.path, test_case.config_location,
                "{}",
                &test_case.name
            );
            assert_eq!(
                source.credentials_file.path, test_case.credentials_location,
                "{}",
                &test_case.name
            )
        } else {
            println!(
                "NOTE: ignoring test case for {} which does not apply to our platform: \n  {}",
                &test_case.platform, &test_case.name
            )
        }
    }

    #[test]
    #[cfg_attr(windows, ignore)]
    fn test_expand_home() {
        let path = "~/.aws/config";
        assert_eq!(
            expand_home(&path, &Some("/user/foo".to_string()))
                .to_str()
                .unwrap(),
            "/user/foo/.aws/config"
        );
    }

    #[test]
    fn homedir_profile_only_windows() {
        // windows specific variables should only be considered when the platform is windows
        let env = Env::from_slice(&[("USERPROFILE", "C:\\Users\\name")]);
        assert_eq!(
            home_dir(&env, Os::Windows),
            Some("C:\\Users\\name".to_string())
        );
        assert_eq!(home_dir(&env, Os::NotWindows), None);
    }

    #[test]
    fn expand_home_no_home() {
        // there is an edge case around expansion when no home directory exists
        // if no home directory can be determined, leave the path as is
        if !cfg!(windows) {
            assert_eq!(expand_home("~/config", &None).to_str().unwrap(), "~/config")
        } else {
            assert_eq!(
                expand_home("~/config", &None).to_str().unwrap(),
                "~\\config"
            )
        }
    }

    /// Test that a linux oriented path expands on windows
    #[test]
    #[cfg_attr(not(windows), ignore)]
    fn test_expand_home_windows() {
        let path = "~/.aws/config";
        assert_eq!(
            expand_home(&path, &Some("C:\\Users\\name".to_string()))
                .to_str()
                .unwrap(),
            "C:\\Users\\name\\.aws\\config"
        );
    }
}

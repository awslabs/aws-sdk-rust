/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::fs_util::{home_dir, Os};
use aws_types::os_shim_internal;
use std::borrow::Cow;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use tracing::Instrument;

/// In-memory source of profile data
pub(super) struct Source {
    /// Contents and path of ~/.aws/config
    pub(super) config_file: File,

    /// Contents and path of ~/.aws/credentials
    pub(super) credentials_file: File,

    /// Profile to use
    ///
    /// Overridden via `$AWS_PROFILE`, defaults to `default`
    pub(super) profile: Cow<'static, str>,
}

/// In-memory configuration file
pub(super) struct File {
    pub(super) path: String,
    pub(super) contents: String,
}

#[derive(Clone, Copy)]
pub(super) enum FileKind {
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
pub(super) async fn load(proc_env: &os_shim_internal::Env, fs: &os_shim_internal::Fs) -> Source {
    let home = home_dir(proc_env, Os::real());
    let config = load_config_file(FileKind::Config, &home, fs, proc_env)
        .instrument(tracing::debug_span!("load_config_file"))
        .await;
    let credentials = load_config_file(FileKind::Credentials, &home, fs, proc_env)
        .instrument(tracing::debug_span!("load_credentials_file"))
        .await;

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
async fn load_config_file(
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
    let expanded = expand_home(path.as_ref(), home_directory, environment);
    if path != expanded.to_string_lossy() {
        tracing::debug!(before = ?path, after = ?expanded, "home directory expanded");
    }
    // read the data at the specified path
    // if the path does not exist, log a warning but pretend it was actually an empty file
    let data = match fs.read_to_end(&expanded).await {
        Ok(data) => data,
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound if path == kind.default_path() => {
                    tracing::debug!(path = %path, "config file not found")
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
    tracing::debug!(path = %path, size = ?data.len(), "config file loaded");
    File {
        // lossy is OK here, the name of this file is just for debugging purposes
        path: expanded.to_string_lossy().into(),
        contents: data,
    }
}

fn expand_home(
    path: impl AsRef<Path>,
    home_dir: &Option<String>,
    environment: &os_shim_internal::Env,
) -> PathBuf {
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
                    // Lambdas don't have home directories and emitting this warning is not helpful
                    // to users running the SDK from within a Lambda. This warning will be silenced
                    // if we determine that that is the case.
                    let is_likely_running_on_a_lambda =
                        check_is_likely_running_on_a_lambda(environment);
                    if !is_likely_running_on_a_lambda {
                        tracing::warn!(
                            "could not determine home directory but home expansion was requested"
                        );
                    }
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

/// Returns true or false based on whether or not this code is likely running inside an AWS Lambda.
/// [Lambdas set many environment variables](https://docs.aws.amazon.com/lambda/latest/dg/configuration-envvars.html#configuration-envvars-runtime)
/// that we can check.
fn check_is_likely_running_on_a_lambda(environment: &aws_types::os_shim_internal::Env) -> bool {
    // AWS_LAMBDA_FUNCTION_NAME – The name of the running Lambda function. Available both in Functions and Extensions
    environment.get("AWS_LAMBDA_FUNCTION_NAME").is_ok()
}

#[cfg(test)]
mod tests {
    use crate::profile::parser::source::{expand_home, load, load_config_file, FileKind};
    use aws_types::os_shim_internal::{Env, Fs};
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fs;

    #[test]
    fn only_expand_home_prefix() {
        // ~ is only expanded as a single component (currently)
        let path = "~aws/config";
        let environment = Env::from_slice(&[]);
        assert_eq!(
            expand_home(&path, &None, &environment).to_str().unwrap(),
            "~aws/config"
        );
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
            check(test)
                .now_or_never()
                .expect("these futures should never poll");
        }
        Ok(())
    }

    use futures_util::FutureExt;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn logs_produced_default() {
        let env = Env::from_slice(&[("HOME", "/user/name")]);
        let mut fs = HashMap::new();
        fs.insert(
            "/user/name/.aws/config".to_string(),
            "[default]\nregion = us-east-1",
        );

        let fs = Fs::from_map(fs);

        let _src = load(&env, &fs).now_or_never();
        assert!(logs_contain("config file loaded"));
        assert!(logs_contain("performing home directory substitution"));
    }

    #[traced_test]
    #[test]
    fn load_config_file_should_not_emit_warning_on_lambda() {
        let env = Env::from_slice(&[("AWS_LAMBDA_FUNCTION_NAME", "someName")]);
        let fs = Fs::from_slice(&[]);

        let _src = load_config_file(FileKind::Config, &None, &fs, &env).now_or_never();
        assert!(!logs_contain(
            "could not determine home directory but home expansion was requested"
        ));
    }

    async fn check(test_case: TestCase) {
        let fs = Fs::real();
        let env = Env::from(test_case.environment);
        let platform_matches = (cfg!(windows) && test_case.platform == "windows")
            || (!cfg!(windows) && test_case.platform != "windows");
        if platform_matches {
            let source = load(&env, &fs).await;
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
        let environment = Env::from_slice(&[]);
        assert_eq!(
            expand_home(&path, &Some("/user/foo".to_string()), &environment)
                .to_str()
                .unwrap(),
            "/user/foo/.aws/config"
        );
    }

    #[test]
    fn expand_home_no_home() {
        let environment = Env::from_slice(&[]);
        // there is an edge case around expansion when no home directory exists
        // if no home directory can be determined, leave the path as is
        if !cfg!(windows) {
            assert_eq!(
                expand_home("~/config", &None, &environment)
                    .to_str()
                    .unwrap(),
                "~/config"
            )
        } else {
            assert_eq!(
                expand_home("~/config", &None, &environment)
                    .to_str()
                    .unwrap(),
                "~\\config"
            )
        }
    }

    /// Test that a linux oriented path expands on windows
    #[test]
    #[cfg_attr(not(windows), ignore)]
    fn test_expand_home_windows() {
        let path = "~/.aws/config";
        let environment = Env::from_slice(&[]);
        assert_eq!(
            expand_home(&path, &Some("C:\\Users\\name".to_string()), &environment)
                .to_str()
                .unwrap(),
            "C:\\Users\\name\\.aws\\config"
        );
    }
}

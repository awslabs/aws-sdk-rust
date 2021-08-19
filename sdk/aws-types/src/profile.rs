/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

mod normalize;
mod parse;
mod source;

// exposed only to remove unused code warnings until the parser side is added
use crate::os_shim_internal::{Env, Fs};
use crate::profile::parse::parse_profile_file;
pub use crate::profile::parse::ProfileParseError;
use crate::profile::source::{FileKind, Source};
use std::borrow::Cow;
use std::collections::HashMap;

/// Read & parse AWS config files
///
/// Loads and parses profile files according to the spec:
///
/// ## Location of Profile Files
/// * The location of the config file will be loaded from `$AWS_CONFIG_FILE` with a fallback to
///   `~/.aws/config`
/// * The location of the credentials file will be loaded from `$AWS_SHARED_CREDENTIALS_FILE` with a
///   fallback to `~/.aws/credentials`
///
/// ## Home directory resolution
/// Home directory resolution is implemented to match the behavior of the CLI & Python. `~` is only
/// used for home directory resolution when it:
/// - Starts the path
/// - Is followed immediately by `/` or a platform specific separator. (On windows, `~/` and `~\` both
///   resolve to the home directory.
///
/// When determining the home directory, the following environment variables are checked:
/// - `$HOME` on all platforms
/// - `$USERPROFILE` on Windows
/// - `$HOMEDRIVE$HOMEPATH` on Windows
///
/// ## Profile file syntax
///
/// Profile files have a general form similar to INI but with a number of quirks and edge cases. These
/// behaviors are largely to match existing parser implementations and these cases are documented in `test-data/profile-parser-tests.json`
/// in this repo.
///
/// ### The config file `~/.aws/config`
/// ```ini
/// # ~/.aws/config
/// [profile default]
/// key = value
///
/// # profiles must begin with `profile`
/// [profile other]
/// key = value2
/// ```
///
/// ### The credentials file `~/.aws/credentials`
/// The main difference is that in ~/.aws/credentials, profiles MUST NOT be prefixed with profile:
/// ```ini
/// [default]
/// aws_access_key_id = 123
///
/// [other]
/// aws_access_key_id = 456
/// ```
pub async fn load(fs: &Fs, env: &Env) -> Result<ProfileSet, ProfileParseError> {
    let source = source::load(&env, &fs).await;
    ProfileSet::parse(source)
}

/// A top-level configuration source containing multiple named profiles
#[derive(Debug, Eq, Clone, PartialEq)]
pub struct ProfileSet {
    profiles: HashMap<String, Profile>,
    selected_profile: Cow<'static, str>,
}

impl ProfileSet {
    /// Create a new Profile set directly from a HashMap
    ///
    /// This method creates a ProfileSet directly from a hashmap with no normalization.
    ///
    /// ## Note
    ///
    /// This is probably not what you want! In general, [`load`](load) should be used instead
    /// because it will perform input normalization. However, for tests which operate on the
    /// normalized profile, this method exists to facilitate easy construction of a ProfileSet
    pub fn new(
        profiles: HashMap<String, HashMap<String, String>>,
        selected_profile: impl Into<Cow<'static, str>>,
    ) -> Self {
        let mut base = ProfileSet::empty();
        base.selected_profile = selected_profile.into();
        for (name, profile) in profiles {
            base.profiles.insert(
                name.clone(),
                Profile::new(
                    name,
                    profile
                        .into_iter()
                        .map(|(k, v)| (k.clone(), Property::new(k, v)))
                        .collect(),
                ),
            );
        }
        base
    }

    /// Retrieves a key-value pair from the currently selected profile
    pub fn get(&self, key: &str) -> Option<&str> {
        self.profiles
            .get(self.selected_profile.as_ref())
            .and_then(|profile| profile.get(key))
    }

    /// Retrieve a named profile from the profile set
    pub fn get_profile(&self, profile_name: &str) -> Option<&Profile> {
        self.profiles.get(profile_name)
    }

    pub fn selected_profile(&self) -> &str {
        self.selected_profile.as_ref()
    }

    /// Returns true if no profiles are contained in this profile set
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    fn parse(source: Source) -> Result<Self, ProfileParseError> {
        let mut base = ProfileSet::empty();
        base.selected_profile = source.profile;

        normalize::merge_in(
            &mut base,
            parse_profile_file(&source.config_file)?,
            FileKind::Config,
        );
        normalize::merge_in(
            &mut base,
            parse_profile_file(&source.credentials_file)?,
            FileKind::Credentials,
        );
        Ok(base)
    }

    fn empty() -> Self {
        Self {
            profiles: Default::default(),
            selected_profile: "default".into(),
        }
    }
}

/// An individual configuration profile
///
/// An AWS config may be composed of a multiple named profiles within a [`ProfileSet`](ProfileSet)
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Profile {
    name: String,
    properties: HashMap<String, Property>,
}

impl Profile {
    pub fn new(name: String, properties: HashMap<String, Property>) -> Self {
        Self { name, properties }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.properties.get(name).map(|prop| prop.value())
    }
}

/// Key-Value property pair
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Property {
    key: String,
    value: String,
}

impl Property {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn new(key: String, value: String) -> Self {
        Property { key, value }
    }
}

#[cfg(test)]
mod test {
    use crate::profile::source::{File, Source};
    use crate::profile::ProfileSet;
    use arbitrary::{Arbitrary, Unstructured};
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fs;
    use tracing_test::traced_test;

    /// Run all tests from profile-parser-tests.json
    ///
    /// These represent the bulk of the test cases and reach effectively 100% coverage
    #[test]
    #[traced_test]
    fn run_tests() -> Result<(), Box<dyn Error>> {
        let tests = fs::read_to_string("test-data/profile-parser-tests.json")?;
        let tests: ParserTests = serde_json::from_str(&tests)?;
        for (i, test) in tests.tests.into_iter().enumerate() {
            eprintln!("test: {}", i);
            check(test);
        }
        Ok(())
    }

    #[test]
    fn empty_source_empty_profile() {
        let source = Source {
            config_file: File {
                path: "~/.aws/config".to_string(),
                contents: "".into(),
            },
            credentials_file: File {
                path: "~/.aws/credentials".to_string(),
                contents: "".into(),
            },
            profile: "default".into(),
        };
        let profile_set = ProfileSet::parse(source).expect("empty profiles are valid");
        assert_eq!(profile_set.is_empty(), true);
    }

    /// Run all tests from the fuzzing corpus to validate coverage
    #[test]
    #[ignore]
    fn run_fuzz_tests() -> Result<(), Box<dyn Error>> {
        let fuzz_corpus = fs::read_dir("fuzz/corpus/profile-parser")?
            .map(|res| res.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        for file in fuzz_corpus {
            let raw = fs::read(file)?;
            let mut unstructured = Unstructured::new(&raw);
            let (conf, creds): (Option<&str>, Option<&str>) =
                Arbitrary::arbitrary(&mut unstructured)?;
            let profile_source = Source {
                config_file: File {
                    path: "~/.aws/config".to_string(),
                    contents: conf.unwrap_or_default().to_string(),
                },
                credentials_file: File {
                    path: "~/.aws/config".to_string(),
                    contents: creds.unwrap_or_default().to_string(),
                },
                profile: "default".into(),
            };
            // don't care if parse fails, just don't panic
            let _ = ProfileSet::parse(profile_source);
        }

        Ok(())
    }

    // for test comparison purposes, flatten a profile into a hashmap
    fn flatten(profile: ProfileSet) -> HashMap<String, HashMap<String, String>> {
        profile
            .profiles
            .into_iter()
            .map(|(_name, profile)| {
                (
                    profile.name,
                    profile
                        .properties
                        .into_iter()
                        .map(|(_, prop)| (prop.key, prop.value))
                        .collect(),
                )
            })
            .collect()
    }

    fn make_source(input: ParserInput) -> Source {
        Source {
            config_file: File {
                path: "~/.aws/config".to_string(),
                contents: input.config_file.unwrap_or_default(),
            },
            credentials_file: File {
                path: "~/.aws/credentials".to_string(),
                contents: input.credentials_file.unwrap_or_default(),
            },
            profile: "default".into(),
        }
    }

    // wrapper to generate nicer errors during test failure
    fn check(test_case: ParserTest) {
        let copy = test_case.clone();
        let parsed = ProfileSet::parse(make_source(test_case.input));
        let res = match (parsed.map(flatten), &test_case.output) {
            (Ok(actual), ParserOutput::Profiles(expected)) if &actual != expected => Err(format!(
                "mismatch:\nExpected: {:#?}\nActual: {:#?}",
                expected, actual
            )),
            (Ok(_), ParserOutput::Profiles(_)) => Ok(()),
            (Err(msg), ParserOutput::ErrorContaining(substr)) => {
                if format!("{}", msg).contains(substr) {
                    Ok(())
                } else {
                    Err(format!("Expected {} to contain {}", msg, substr))
                }
            }
            (Ok(output), ParserOutput::ErrorContaining(err)) => Err(format!(
                "expected an error: {} but parse succeeded:\n{:#?}",
                err, output
            )),
            (Err(err), ParserOutput::Profiles(_expected)) => {
                Err(format!("Expected to succeed but got: {}", err))
            }
        };
        if let Err(e) = res {
            eprintln!("Test case failed: {:#?}", copy);
            eprintln!("failure: {}", e);
            panic!("test failed")
        }
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ParserTests {
        tests: Vec<ParserTest>,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    struct ParserTest {
        name: String,
        input: ParserInput,
        output: ParserOutput,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    enum ParserOutput {
        Profiles(HashMap<String, HashMap<String, String>>),
        ErrorContaining(String),
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    struct ParserInput {
        config_file: Option<String>,
        credentials_file: Option<String>,
    }
}

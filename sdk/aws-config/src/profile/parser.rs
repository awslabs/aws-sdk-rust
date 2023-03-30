/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::profile::parser::parse::parse_profile_file;
use crate::profile::parser::source::Source;
use crate::profile::profile_file::ProfileFiles;
use aws_types::os_shim_internal::{Env, Fs};
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;

pub use self::parse::ProfileParseError;

mod normalize;
mod parse;
mod source;

/// Read & parse AWS config files
///
/// Loads AWS config file from the filesystem, parses them, and converts them into a [`ProfileSet`](ProfileSet).
///
/// Although the basic behavior is straightforward, there are number of nuances to maintain backwards
/// compatibility with other SDKs enumerated below.
///
#[doc = include_str!("location_of_profile_files.md")]
///
/// ## Profile file syntax
///
/// Profile files have a form similar to `.ini` but with a several edge cases. These behaviors exist
/// to match existing parser implementations, ensuring consistent behavior across AWS SDKs. These
/// cases fully enumerated in `test-data/profile-parser-tests.json`.
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
pub async fn load(
    fs: &Fs,
    env: &Env,
    profile_files: &ProfileFiles,
    selected_profile_override: Option<Cow<'static, str>>,
) -> Result<ProfileSet, ProfileFileLoadError> {
    let mut source = source::load(env, fs, profile_files).await?;
    if let Some(profile) = selected_profile_override {
        source.profile = profile;
    }

    Ok(ProfileSet::parse(source)?)
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
    /// This method creates a ProfileSet directly from a hashmap with no normalization for test purposes.
    #[cfg(test)]
    pub(crate) fn new(
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

    /// Retrieves a named profile from the profile set
    pub fn get_profile(&self, profile_name: &str) -> Option<&Profile> {
        self.profiles.get(profile_name)
    }

    /// Returns the name of the currently selected profile
    pub fn selected_profile(&self) -> &str {
        self.selected_profile.as_ref()
    }

    /// Returns true if no profiles are contained in this profile set
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    /// Returns the names of the profiles in this profile set
    pub fn profiles(&self) -> impl Iterator<Item = &str> {
        self.profiles.keys().map(String::as_ref)
    }

    fn parse(source: Source) -> Result<Self, ProfileParseError> {
        let mut base = ProfileSet::empty();
        base.selected_profile = source.profile;

        for file in source.files {
            normalize::merge_in(&mut base, parse_profile_file(&file)?, file.kind);
        }
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
    /// Create a new profile
    pub fn new(name: String, properties: HashMap<String, Property>) -> Self {
        Self { name, properties }
    }

    /// The name of this profile
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the property named `name`
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
    /// Value of this property
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Name of this property
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Creates a new property
    pub fn new(key: String, value: String) -> Self {
        Property { key, value }
    }
}

/// Failed to read or parse the profile file(s)
#[derive(Debug, Clone)]
pub enum ProfileFileLoadError {
    /// The profile could not be parsed
    #[non_exhaustive]
    ParseError(ProfileParseError),

    /// Attempt to read the AWS config file (`~/.aws/config` by default) failed with a filesystem error.
    #[non_exhaustive]
    CouldNotReadFile(CouldNotReadProfileFile),
}

impl Display for ProfileFileLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileFileLoadError::ParseError(_err) => {
                write!(f, "could not parse profile file")
            }
            ProfileFileLoadError::CouldNotReadFile(err) => {
                write!(f, "could not read file `{}`", err.path.display())
            }
        }
    }
}

impl Error for ProfileFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProfileFileLoadError::ParseError(err) => Some(err),
            ProfileFileLoadError::CouldNotReadFile(details) => Some(&details.cause),
        }
    }
}

impl From<ProfileParseError> for ProfileFileLoadError {
    fn from(err: ProfileParseError) -> Self {
        ProfileFileLoadError::ParseError(err)
    }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct CouldNotReadProfileFile {
    pub(crate) path: PathBuf,
    pub(crate) cause: Arc<std::io::Error>,
}

#[cfg(test)]
mod test {
    use crate::profile::parser::source::{File, Source};
    use crate::profile::profile_file::ProfileFileKind;
    use crate::profile::ProfileSet;
    use arbitrary::{Arbitrary, Unstructured};
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fs;
    use tracing_test::traced_test;

    /// Run all tests from `test-data/profile-parser-tests.json`
    ///
    /// These represent the bulk of the test cases and reach 100% coverage of the parser.
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
        let source = make_source(ParserInput {
            config_file: Some("".to_string()),
            credentials_file: Some("".to_string()),
        });

        let profile_set = ProfileSet::parse(source).expect("empty profiles are valid");
        assert!(profile_set.is_empty());
    }

    #[test]
    fn profile_names_are_exposed() {
        let source = make_source(ParserInput {
            config_file: Some("[profile foo]\n[profile bar]".to_string()),
            credentials_file: Some("".to_string()),
        });

        let profile_set = ProfileSet::parse(source).expect("profiles loaded");

        let mut profile_names: Vec<_> = profile_set.profiles().collect();
        profile_names.sort();
        assert_eq!(profile_names, vec!["bar", "foo"]);
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
                files: vec![
                    File {
                        kind: ProfileFileKind::Config,
                        path: Some("~/.aws/config".to_string()),
                        contents: conf.unwrap_or_default().to_string(),
                    },
                    File {
                        kind: ProfileFileKind::Credentials,
                        path: Some("~/.aws/credentials".to_string()),
                        contents: creds.unwrap_or_default().to_string(),
                    },
                ],
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
            .into_values()
            .map(|profile| {
                (
                    profile.name,
                    profile
                        .properties
                        .into_values()
                        .map(|prop| (prop.key, prop.value))
                        .collect(),
                )
            })
            .collect()
    }

    fn make_source(input: ParserInput) -> Source {
        Source {
            files: vec![
                File {
                    kind: ProfileFileKind::Config,
                    path: Some("~/.aws/config".to_string()),
                    contents: input.config_file.unwrap_or_default(),
                },
                File {
                    kind: ProfileFileKind::Credentials,
                    path: Some("~/.aws/credentials".to_string()),
                    contents: input.credentials_file.unwrap_or_default(),
                },
            ],
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
        _name: String,
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

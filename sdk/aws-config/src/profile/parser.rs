/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use self::parse::parse_profile_file;
use self::section::{Section, SsoSession};
use self::source::Source;
use super::profile_file::ProfileFiles;
use crate::profile::parser::section::Properties;
use aws_types::os_shim_internal::{Env, Fs};
use std::borrow::Cow;
use std::collections::HashMap;

pub use self::error::ProfileFileLoadError;
pub use self::parse::ProfileParseError;
pub use self::section::Profile;
pub use self::section::Property;

pub(crate) use self::section::PropertiesKey;

mod error;
mod normalize;
mod parse;
mod section;
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
    sso_sessions: HashMap<String, SsoSession>,
    other_sections: Properties,
}

impl ProfileSet {
    /// Create a new Profile set directly from a HashMap
    ///
    /// This method creates a ProfileSet directly from a hashmap with no normalization for test purposes.
    #[cfg(test)]
    pub(crate) fn new(
        profiles: HashMap<String, HashMap<String, String>>,
        selected_profile: impl Into<Cow<'static, str>>,
        sso_sessions: HashMap<String, HashMap<String, String>>,
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
        for (name, session) in sso_sessions {
            base.sso_sessions.insert(
                name.clone(),
                SsoSession::new(
                    name,
                    session
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

    /// Returns the names of the profiles in this config
    pub fn profiles(&self) -> impl Iterator<Item = &str> {
        self.profiles.keys().map(String::as_ref)
    }

    /// Returns the names of the SSO sessions in this config
    pub fn sso_sessions(&self) -> impl Iterator<Item = &str> {
        self.sso_sessions.keys().map(String::as_ref)
    }

    /// Retrieves a named SSO session from the config
    pub(crate) fn sso_session(&self, name: &str) -> Option<&SsoSession> {
        self.sso_sessions.get(name)
    }

    /// Returns a struct allowing access to other sections in the profile config
    #[allow(dead_code)] // Leaving this hidden for now.
    pub(crate) fn other_sections(&self) -> &Properties {
        &self.other_sections
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
            sso_sessions: Default::default(),
            other_sections: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::section::Section;
    use super::source::{File, Source};
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
    #[derive(Debug)]
    struct FlattenedProfileSet {
        profiles: HashMap<String, HashMap<String, String>>,
        sso_sessions: HashMap<String, HashMap<String, String>>,
    }
    fn flatten(config: ProfileSet) -> FlattenedProfileSet {
        FlattenedProfileSet {
            profiles: flatten_sections(config.profiles.values().map(|p| p as _)),
            sso_sessions: flatten_sections(config.sso_sessions.values().map(|s| s as _)),
        }
    }
    fn flatten_sections<'a>(
        sections: impl Iterator<Item = &'a dyn Section>,
    ) -> HashMap<String, HashMap<String, String>> {
        sections
            .map(|section| {
                (
                    section.name().to_string(),
                    section
                        .properties()
                        .values()
                        .map(|prop| (prop.key().to_owned(), prop.value().to_owned()))
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
            (
                Ok(FlattenedProfileSet {
                    profiles: actual_profiles,
                    sso_sessions: actual_sso_sessions,
                }),
                ParserOutput::Config {
                    profiles,
                    sso_sessions,
                },
            ) => {
                if profiles != &actual_profiles {
                    Err(format!(
                        "mismatched profiles:\nExpected: {profiles:#?}\nActual: {actual_profiles:#?}",
                    ))
                } else if sso_sessions != &actual_sso_sessions {
                    Err(format!(
                        "mismatched sso_sessions:\nExpected: {sso_sessions:#?}\nActual: {actual_sso_sessions:#?}",
                    ))
                } else {
                    Ok(())
                }
            }
            (Err(msg), ParserOutput::ErrorContaining(substr)) => {
                if format!("{}", msg).contains(substr) {
                    Ok(())
                } else {
                    Err(format!("Expected {} to contain {}", msg, substr))
                }
            }
            (Ok(output), ParserOutput::ErrorContaining(err)) => Err(format!(
                "expected an error: {err} but parse succeeded:\n{output:#?}",
            )),
            (Err(err), ParserOutput::Config { .. }) => {
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
        Config {
            profiles: HashMap<String, HashMap<String, String>>,
            #[serde(default)]
            sso_sessions: HashMap<String, HashMap<String, String>>,
        },
        ErrorContaining(String),
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    struct ParserInput {
        config_file: Option<String>,
        credentials_file: Option<String>,
    }
}

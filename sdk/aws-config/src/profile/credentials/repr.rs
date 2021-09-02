/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Flattened Representation of an AssumeRole chain
//!
//! Assume Role credentials in profile files can chain together credentials from multiple
//! different providers with subsequent credentials being used to configure subsequent providers.
//!
//! This module can parse and resolve the profile chain into a flattened representation with
//! 1-credential-per row (as opposed to a direct profile file representation which can combine
//! multiple actions into the same profile).

use crate::profile::credentials::ProfileFileError;
use crate::profile::{Profile, ProfileSet};
use aws_types::Credentials;

/// Chain of Profile Providers
///
/// Within a profile file, a chain of providers is produced. Starting with a base provider,
/// subsequent providers use the credentials from previous providers to perform their task.
///
/// ProfileChain is a direct representation of the Profile. It can contain named providers
/// that don't actually have implementations.
#[derive(Debug)]
pub struct ProfileChain<'a> {
    pub(crate) base: BaseProvider<'a>,
    pub(crate) chain: Vec<RoleArn<'a>>,
}

impl<'a> ProfileChain<'a> {
    pub fn base(&self) -> &BaseProvider<'a> {
        &self.base
    }

    pub fn chain(&self) -> &[RoleArn<'a>] {
        &self.chain.as_slice()
    }
}

/// A base member of the profile chain
///
/// Base providers do not require input credentials to provide their own credentials,
/// eg. IMDS, ECS, Environment variables
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum BaseProvider<'a> {
    /// A profile that specifies a named credential source
    /// Eg: `credential_source = Ec2InstanceMetadata`
    ///
    /// The following profile produces two separate `ProfileProvider` rows:
    /// 1. `BaseProvider::NamedSource("Ec2InstanceMetadata")`
    /// 2. `RoleArn { role_arn: "...", ... }
    /// ```ini
    /// [profile assume-role]
    /// role_arn = arn:aws:iam::123456789:role/MyRole
    /// credential_source = Ec2InstanceMetadata
    /// ```
    NamedSource(&'a str),

    /// A profile with explicitly configured access keys
    ///
    /// Example
    /// ```ini
    /// [profile C]
    /// aws_access_key_id = abc123
    /// aws_secret_access_key = def456
    /// ```
    AccessKey(Credentials),

    WebIdentityTokenRole {
        role_arn: &'a str,
        web_identity_token_file: &'a str,
        session_name: Option<&'a str>,
    }, // TODO: add SSO support
       /*
       /// An SSO Provider
       Sso {
           sso_account_id: &'a str,
           sso_region: &'a str,
           sso_role_name: &'a str,
           sso_start_url: &'a str,
       },
        */
}

/// A profile that specifies a role to assume
///
/// A RoleArn can only be created from either a profile with `source_profile`
/// or one with `credential_source`.
#[derive(Debug)]
pub struct RoleArn<'a> {
    /// Role to assume
    pub role_arn: &'a str,
    /// external_id parameter to pass to the assume role provider
    pub external_id: Option<&'a str>,

    /// session name parameter to pass to the assume role provider
    pub session_name: Option<&'a str>,
}

/// Resolve a ProfileChain from a ProfileSet or return an error
pub fn resolve_chain<'a>(
    profile_set: &'a ProfileSet,
    profile_override: Option<&str>,
) -> Result<ProfileChain<'a>, ProfileFileError> {
    if profile_set.is_empty() {
        return Err(ProfileFileError::NoProfilesDefined);
    }
    let mut source_profile_name =
        profile_override.unwrap_or_else(|| profile_set.selected_profile());
    let mut visited_profiles = vec![];
    let mut chain = vec![];
    let base = loop {
        let profile = profile_set.get_profile(source_profile_name).ok_or(
            ProfileFileError::MissingProfile {
                profile: source_profile_name.into(),
                message: format!(
                    "could not find source profile {} referenced from {}",
                    source_profile_name,
                    visited_profiles.last().unwrap_or(&"the root profile")
                )
                .into(),
            },
        )?;
        if visited_profiles.contains(&source_profile_name) {
            return Err(ProfileFileError::CredentialLoop {
                profiles: visited_profiles
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
                next: source_profile_name.to_string(),
            });
        }
        visited_profiles.push(&source_profile_name);
        // After the first item in the chain, we will prioritize static credentials if they exist
        if visited_profiles.len() > 1 {
            let try_static = static_creds_from_profile(&profile);
            if let Ok(static_credentials) = try_static {
                break BaseProvider::AccessKey(static_credentials);
            }
        }
        let next_profile = match chain_provider(&profile) {
            // this provider wasn't a chain provider, reload it as a base provider
            None => {
                break base_provider(profile)?;
            }
            Some(result) => {
                let (chain_profile, next) = result?;
                chain.push(chain_profile);
                next
            }
        };
        match next_profile {
            NextProfile::SelfReference => {
                // self referential profile, don't go through the loop because it will error
                // on the infinite loop check. Instead, reload this profile as a base profile
                // and exit.
                break base_provider(profile)?;
            }
            NextProfile::Named(name) => source_profile_name = name,
        }
    };
    chain.reverse();
    Ok(ProfileChain { base, chain })
}

mod role {
    pub const ROLE_ARN: &str = "role_arn";
    pub const EXTERNAL_ID: &str = "external_id";
    pub const SESSION_NAME: &str = "role_session_name";

    pub const CREDENTIAL_SOURCE: &str = "credential_source";
    pub const SOURCE_PROFILE: &str = "source_profile";
}

mod web_identity_token {
    pub const TOKEN_FILE: &str = "web_identity_token_file";
}

mod static_credentials {
    pub const AWS_ACCESS_KEY_ID: &str = "aws_access_key_id";
    pub const AWS_SECRET_ACCESS_KEY: &str = "aws_secret_access_key";
    pub const AWS_SESSION_TOKEN: &str = "aws_session_token";
}
const PROVIDER_NAME: &str = "ProfileFile";

fn base_provider(profile: &Profile) -> Result<BaseProvider, ProfileFileError> {
    // the profile must define either a `CredentialsSource` or a concrete set of access keys
    match profile.get(role::CREDENTIAL_SOURCE) {
        Some(source) => Ok(BaseProvider::NamedSource(source)),
        None => web_identity_token_from_profile(profile)
            .unwrap_or_else(|| Ok(BaseProvider::AccessKey(static_creds_from_profile(profile)?))),
    }
}

enum NextProfile<'a> {
    SelfReference,
    Named(&'a str),
}

fn chain_provider(profile: &Profile) -> Option<Result<(RoleArn, NextProfile), ProfileFileError>> {
    let role_provider = role_arn_from_profile(&profile)?;
    let (source_profile, credential_source) = (
        profile.get(role::SOURCE_PROFILE),
        profile.get(role::CREDENTIAL_SOURCE),
    );
    let profile = match (source_profile, credential_source) {
        (Some(_), Some(_)) => Err(ProfileFileError::InvalidCredentialSource {
            profile: profile.name().to_string(),
            message: "profile contained both source_profile and credential_source. \
                Only one or the other can be defined"
                .into(),
        }),
        (None, None) => Err(ProfileFileError::InvalidCredentialSource {
            profile: profile.name().to_string(),
            message:
                "profile must contain source_profile or credentials_source but neither were defined"
                    .into(),
        }),
        (Some(source_profile), None) if source_profile == profile.name() => {
            Ok((role_provider, NextProfile::SelfReference))
        }

        (Some(source_profile), None) => Ok((role_provider, NextProfile::Named(source_profile))),
        // we want to loop back into this profile and pick up the credential source
        (None, Some(_credential_source)) => Ok((role_provider, NextProfile::SelfReference)),
    };
    Some(profile)
}

fn role_arn_from_profile(profile: &Profile) -> Option<RoleArn> {
    // Web Identity Tokens are root providers, not chained roles
    if profile.get(web_identity_token::TOKEN_FILE).is_some() {
        return None;
    }
    let role_arn = profile.get(role::ROLE_ARN)?;
    let session_name = profile.get(role::SESSION_NAME);
    let external_id = profile.get(role::EXTERNAL_ID);
    Some(RoleArn {
        role_arn,
        external_id,
        session_name,
    })
}

fn web_identity_token_from_profile(
    profile: &Profile,
) -> Option<Result<BaseProvider, ProfileFileError>> {
    let session_name = profile.get(role::SESSION_NAME);
    match (
        profile.get(role::ROLE_ARN),
        profile.get(web_identity_token::TOKEN_FILE),
    ) {
        (Some(role_arn), Some(token_file)) => Some(Ok(BaseProvider::WebIdentityTokenRole {
            role_arn,
            web_identity_token_file: token_file,
            session_name,
        })),
        (None, None) => None,
        (Some(_role_arn), None) => None,
        (None, Some(_token_file)) => Some(Err(ProfileFileError::InvalidCredentialSource {
            profile: profile.name().to_string(),
            message: "`web_identity_token_file` was specified but `role_arn` was missing".into(),
        })),
    }
}

/// Load static credentials from a profile
///
/// Example:
/// ```ini
/// [profile B]
/// aws_access_key_id = abc123
/// aws_secret_access_key = def456
/// ```
fn static_creds_from_profile(profile: &Profile) -> Result<Credentials, ProfileFileError> {
    use static_credentials::*;
    let access_key = profile.get(AWS_ACCESS_KEY_ID);
    let secret_key = profile.get(AWS_SECRET_ACCESS_KEY);
    let session_token = profile.get(AWS_SESSION_TOKEN);
    if let (None, None, None) = (access_key, secret_key, session_token) {
        return Err(ProfileFileError::MissingCredentialSource {
            profile: profile.name().to_string(),
            message: "expected `aws_access_key_id` and `aws_secret_access_key` to be defined"
                .into(),
        });
    }
    let access_key = access_key.ok_or_else(|| ProfileFileError::InvalidCredentialSource {
        profile: profile.name().to_string(),
        message: "profile missing aws_access_key_id".into(),
    })?;
    let secret_key = secret_key.ok_or_else(|| ProfileFileError::InvalidCredentialSource {
        profile: profile.name().to_string(),
        message: "profile missing aws_secret_access_key".into(),
    })?;
    Ok(Credentials::new(
        access_key,
        secret_key,
        session_token.map(|s| s.to_string()),
        None,
        PROVIDER_NAME,
    ))
}

#[cfg(test)]
mod tests {
    use crate::profile::credentials::repr::{resolve_chain, BaseProvider, ProfileChain};
    use crate::profile::ProfileSet;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fs;

    #[test]
    fn run_test_cases() -> Result<(), Box<dyn Error>> {
        let test_cases: Vec<TestCase> =
            serde_json::from_str(&fs::read_to_string("./test-data/assume-role-tests.json")?)?;
        for test_case in test_cases {
            print!("checking: {}...", test_case.docs);
            check(test_case);
            println!("ok")
        }
        Ok(())
    }

    fn check(test_case: TestCase) {
        let source = ProfileSet::new(test_case.input.profile, test_case.input.selected_profile);
        let actual = resolve_chain(&source, None);
        let expected = test_case.output;
        match (expected, actual) {
            (TestOutput::Error(s), Err(e)) => assert!(
                format!("{}", e).contains(&s),
                "expected {} to contain `{}`",
                e,
                s
            ),
            (TestOutput::ProfileChain(expected), Ok(actual)) => {
                assert_eq!(to_test_output(actual), expected)
            }
            (expected, actual) => panic!(
                "error/success mismatch. Expected:\n {:?}\nActual:\n {:?}",
                &expected, actual
            ),
        }
    }

    #[derive(Deserialize)]
    struct TestCase {
        docs: String,
        input: TestInput,
        output: TestOutput,
    }

    #[derive(Deserialize)]
    struct TestInput {
        profile: HashMap<String, HashMap<String, String>>,
        selected_profile: String,
    }

    fn to_test_output(profile_chain: ProfileChain) -> Vec<Provider> {
        let mut output = vec![];
        match profile_chain.base {
            BaseProvider::NamedSource(name) => output.push(Provider::NamedSource(name.into())),
            BaseProvider::AccessKey(creds) => output.push(Provider::AccessKey {
                access_key_id: creds.access_key_id().into(),
                secret_access_key: creds.secret_access_key().into(),
                session_token: creds.session_token().map(|tok| tok.to_string()),
            }),
            BaseProvider::WebIdentityTokenRole {
                role_arn,
                web_identity_token_file,
                session_name,
            } => output.push(Provider::WebIdentityToken {
                role_arn: role_arn.into(),
                web_identity_token_file: web_identity_token_file.into(),
                role_session_name: session_name.map(|sess| sess.to_string()),
            }),
        };
        for role in profile_chain.chain {
            output.push(Provider::AssumeRole {
                role_arn: role.role_arn.into(),
                external_id: role.external_id.map(ToString::to_string),
                role_session_name: role.session_name.map(ToString::to_string),
            })
        }
        output
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum TestOutput {
        ProfileChain(Vec<Provider>),
        Error(String),
    }

    #[derive(Deserialize, Debug, Eq, PartialEq)]
    enum Provider {
        AssumeRole {
            role_arn: String,
            external_id: Option<String>,
            role_session_name: Option<String>,
        },
        AccessKey {
            access_key_id: String,
            secret_access_key: String,
            session_token: Option<String>,
        },
        NamedSource(String),
        WebIdentityToken {
            role_arn: String,
            web_identity_token_file: String,
            role_session_name: Option<String>,
        },
    }
}

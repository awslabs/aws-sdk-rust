/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::profile::parser::{
    parse::{RawProfileSet, WHITESPACE},
    Section, SsoSession,
};
use crate::profile::profile_file::ProfileFileKind;
use crate::profile::{Profile, ProfileSet, Property};
use std::borrow::Cow;
use std::collections::HashMap;

const DEFAULT: &str = "default";
const PROFILE_PREFIX: &str = "profile";
const SSO_SESSION_PREFIX: &str = "sso-session";

#[derive(Eq, PartialEq, Hash, Debug)]
enum SectionKey<'a> {
    /// `[default]` or `[profile default]`
    Default {
        /// True when it is `[profile default]`
        prefixed: bool,
    },
    /// `[profile name]` or `[name]`
    Profile {
        name: Cow<'a, str>,
        /// True if prefixed with `profile`.
        prefixed: bool,
    },
    /// `[sso-session name]`
    SsoSession { name: Cow<'a, str> },
}

impl<'a> SectionKey<'a> {
    fn parse(input: &str) -> SectionKey<'_> {
        let input = input.trim_matches(WHITESPACE);
        if input == DEFAULT {
            return SectionKey::Default { prefixed: false };
        } else if let Some((prefix, suffix)) = input.split_once(WHITESPACE) {
            let suffix = suffix.trim();
            if prefix == PROFILE_PREFIX {
                if suffix == "default" {
                    return SectionKey::Default { prefixed: true };
                } else {
                    return SectionKey::Profile {
                        name: suffix.into(),
                        prefixed: true,
                    };
                }
            } else if prefix == SSO_SESSION_PREFIX {
                return SectionKey::SsoSession {
                    name: suffix.into(),
                };
            }
        }

        SectionKey::Profile {
            name: input.into(),
            prefixed: false,
        }
    }

    /// Validate a SectionKey for a given file key
    ///
    /// 1. `name` must ALWAYS be a valid identifier
    /// 2. For Config files, the profile must either be `default` or it must have a profile prefix
    /// 3. For credentials files, the profile name MUST NOT have a profile prefix
    /// 4. Only config files can have sso-session sections
    fn valid_for(self, kind: ProfileFileKind) -> Result<Self, String> {
        match &self {
            SectionKey::Default { .. } => Ok(self),
            SectionKey::Profile { name, prefixed } => {
                if validate_identifier(name).is_err() {
                    return Err(format!(
                        "profile `{}` ignored because `{}` was not a valid identifier",
                        name, name
                    ));
                }
                match (kind, prefixed) {
                    (ProfileFileKind::Config, false) => Err(format!("profile `{}` ignored because config profiles must be of the form `[profile <name>]`", name)),
                    (ProfileFileKind::Credentials, true) => Err(format!("profile `{}` ignored because credential profiles must NOT begin with `profile`", name)),
                    _ => Ok(self)
                }
            }
            SectionKey::SsoSession { name } => {
                if validate_identifier(name).is_err() {
                    return Err(format!(
                        "sso-session `{}` ignored because `{}` was not a valid identifier",
                        name, name
                    ));
                }
                if let ProfileFileKind::Config = kind {
                    Ok(self)
                } else {
                    Err(format!(
                        "sso-session `{}` ignored sso-sessions must be in the AWS config file rather than the credentials file",
                        name
                    ))
                }
            }
        }
    }
}

/// Normalize a raw profile into a `MergedProfile`
///
/// This function follows the following rules, codified in the tests & the reference Java implementation
/// - When the profile is a config file, strip `profile` and trim whitespace (`profile foo` => `foo`)
/// - Profile names are validated (see `validate_profile_name`)
/// - A profile named `profile default` takes priority over a profile named `default`.
/// - Profiles with identical names are merged
pub(super) fn merge_in(
    base: &mut ProfileSet,
    raw_profile_set: RawProfileSet<'_>,
    kind: ProfileFileKind,
) {
    // parse / validate sections
    let validated_sections = raw_profile_set
        .into_iter()
        .map(|(name, properties)| (SectionKey::parse(name).valid_for(kind), properties));

    // remove invalid profiles & emit warning
    // valid_sections contains only valid profiles but it may contain `[profile default]` and `[default]`
    // which must be filtered later
    let valid_sections = validated_sections
        .filter_map(|(name, properties)| match name {
            Ok(section_key) => Some((section_key, properties)),
            Err(err_str) => {
                tracing::warn!("{}", err_str);
                None
            }
        })
        .collect::<Vec<_>>();
    // if a `[profile default]` exists then we should ignore `[default]`
    let ignore_unprefixed_default = valid_sections
        .iter()
        .any(|(section_key, _)| matches!(section_key, SectionKey::Default { prefixed: true }));

    for (section_key, raw_profile) in valid_sections {
        // When normalizing profiles, profiles should be merged. However, `[profile default]` and
        // `[default]` are considered two separate profiles. Furthermore, `[profile default]` fully
        // replaces any contents of `[default]`!
        if ignore_unprefixed_default
            && matches!(section_key, SectionKey::Default { prefixed: false })
        {
            tracing::warn!("profile `default` ignored because `[profile default]` was found which takes priority");
            continue;
        }
        let section: &mut dyn Section = match section_key {
            SectionKey::Default { .. } => base
                .profiles
                .entry("default".to_string())
                .or_insert_with(|| Profile::new("default", Default::default())),
            SectionKey::Profile { name, .. } => base
                .profiles
                .entry(name.to_string())
                .or_insert_with(|| Profile::new(name, Default::default())),
            SectionKey::SsoSession { name } => base
                .sso_sessions
                .entry(name.to_string())
                .or_insert_with(|| SsoSession::new(name, Default::default())),
        };
        merge_into_base(section, raw_profile)
    }
}

fn merge_into_base(target: &mut dyn Section, profile: HashMap<Cow<'_, str>, Cow<'_, str>>) {
    for (k, v) in profile {
        match validate_identifier(k.as_ref()) {
            Ok(k) => {
                target.insert(k.to_owned(), Property::new(k.to_owned(), v.into()));
            }
            Err(_) => {
                tracing::warn!(profile = %target.name(), key = ?k, "key ignored because `{}` was not a valid identifier", k);
            }
        }
    }
}

/// Validate that a string is a valid identifier
///
/// Identifiers must match `[A-Za-z0-9_\-/.%@:\+]+`
fn validate_identifier(input: &str) -> Result<&str, ()> {
    input
        .chars()
        .all(|ch| {
            ch.is_ascii_alphanumeric()
                || ['_', '-', '/', '.', '%', '@', ':', '+']
                    .iter()
                    .any(|c| *c == ch)
        })
        .then_some(input)
        .ok_or(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::parser::{normalize::validate_identifier, Section};
    use crate::profile::parser::{normalize::SectionKey, parse::RawProfileSet};
    use crate::profile::profile_file::ProfileFileKind;
    use crate::profile::ProfileSet;
    use std::borrow::Cow;
    use std::collections::HashMap;
    use tracing_test::traced_test;

    #[test]
    fn section_key_parsing() {
        assert_eq!(
            SectionKey::Default { prefixed: false },
            SectionKey::parse("default"),
        );
        assert_eq!(
            SectionKey::Default { prefixed: false },
            SectionKey::parse("   default "),
        );
        assert_eq!(
            SectionKey::Default { prefixed: true },
            SectionKey::parse("profile default"),
        );
        assert_eq!(
            SectionKey::Default { prefixed: true },
            SectionKey::parse(" profile   default "),
        );

        assert_eq!(
            SectionKey::Profile {
                name: "name".into(),
                prefixed: true
            },
            SectionKey::parse("profile name"),
        );
        assert_eq!(
            SectionKey::Profile {
                name: "name".into(),
                prefixed: false
            },
            SectionKey::parse("name"),
        );
        assert_eq!(
            SectionKey::Profile {
                name: "name".into(),
                prefixed: true
            },
            SectionKey::parse("profile\tname"),
        );
        assert_eq!(
            SectionKey::Profile {
                name: "name".into(),
                prefixed: true
            },
            SectionKey::parse("profile     name  "),
        );
        assert_eq!(
            SectionKey::Profile {
                name: "profilename".into(),
                prefixed: false
            },
            SectionKey::parse("profilename"),
        );
        assert_eq!(
            SectionKey::Profile {
                name: "whitespace".into(),
                prefixed: false
            },
            SectionKey::parse("   whitespace   "),
        );

        assert_eq!(
            SectionKey::SsoSession { name: "foo".into() },
            SectionKey::parse("sso-session foo"),
        );
        assert_eq!(
            SectionKey::SsoSession { name: "foo".into() },
            SectionKey::parse("sso-session\tfoo "),
        );
        assert_eq!(
            SectionKey::Profile {
                name: "sso-sessionfoo".into(),
                prefixed: false
            },
            SectionKey::parse("sso-sessionfoo"),
        );
        assert_eq!(
            SectionKey::Profile {
                name: "sso-session".into(),
                prefixed: false
            },
            SectionKey::parse("sso-session "),
        );
    }

    #[test]
    fn test_validate_identifier() {
        assert_eq!(
            Ok("some-thing:long/the_one%only.foo@bar+"),
            validate_identifier("some-thing:long/the_one%only.foo@bar+")
        );
        assert_eq!(Err(()), validate_identifier("foo!bar"));
    }

    #[test]
    #[traced_test]
    fn ignored_key_generates_warning() {
        let mut profile: RawProfileSet<'_> = HashMap::new();
        profile.insert("default", {
            let mut out = HashMap::new();
            out.insert(Cow::Borrowed("invalid key"), "value".into());
            out
        });
        let mut base = ProfileSet::empty();
        merge_in(&mut base, profile, ProfileFileKind::Config);
        assert!(base
            .get_profile("default")
            .expect("contains default profile")
            .is_empty());
        assert!(logs_contain(
            "key ignored because `invalid key` was not a valid identifier"
        ));
    }

    #[test]
    #[traced_test]
    fn invalid_profile_generates_warning() {
        let mut profile: RawProfileSet<'_> = HashMap::new();
        profile.insert("foo", HashMap::new());
        merge_in(&mut ProfileSet::empty(), profile, ProfileFileKind::Config);
        assert!(logs_contain("profile `foo` ignored"));
    }
}

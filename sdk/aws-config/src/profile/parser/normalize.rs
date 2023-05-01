/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::profile::parser::parse::{RawProfileSet, WHITESPACE};
use crate::profile::profile_file::ProfileFileKind;
use crate::profile::{Profile, ProfileSet, Property};
use std::borrow::Cow;
use std::collections::HashMap;

const DEFAULT: &str = "default";
const PROFILE_PREFIX: &str = "profile";

#[derive(Eq, PartialEq, Hash, Debug)]
struct ProfileName<'a> {
    name: &'a str,
    has_profile_prefix: bool,
}

impl ProfileName<'_> {
    fn parse(input: &str) -> ProfileName<'_> {
        let input = input.trim_matches(WHITESPACE);
        let (name, has_profile_prefix) = match input.strip_prefix(PROFILE_PREFIX) {
            // profilefoo isn't considered as having the profile prefix
            Some(stripped) if stripped.starts_with(WHITESPACE) => (stripped.trim(), true),
            _ => (input, false),
        };
        ProfileName {
            name,
            has_profile_prefix,
        }
    }

    /// Validate a ProfileName for a given file key
    ///
    /// 1. `name` must ALWAYS be a valid identifier
    /// 2. For Config files, the profile must either be `default` or it must have a profile prefix
    /// 3. For credentials files, the profile name MUST NOT have a profile prefix
    fn valid_for(self, kind: ProfileFileKind) -> Result<Self, String> {
        if validate_identifier(self.name).is_err() {
            return Err(format!(
                "profile `{}` ignored because `{}` was not a valid identifier",
                &self.name, &self.name
            ));
        }
        match (self.name, kind, self.has_profile_prefix) {
            (_, ProfileFileKind::Config, true) => Ok(self),
            (DEFAULT, ProfileFileKind::Config, false) => Ok(self),
            (_not_default, ProfileFileKind::Config, false) => Err(format!(
                "profile `{}` ignored because config profiles must be of the form `[profile <name>]`",
                self.name
            )),
            (_, ProfileFileKind::Credentials, true) => Err(format!(
                "profile `{}` ignored because credential profiles must NOT begin with `profile`",
                self.name
            )),
            (_, ProfileFileKind::Credentials, false) => Ok(self),
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
    // parse / validate profile names
    let validated_profiles = raw_profile_set
        .into_iter()
        .map(|(name, profile)| (ProfileName::parse(name).valid_for(kind), profile));

    // remove invalid profiles & emit warning
    // valid_profiles contains only valid profiles but it may contain `[profile default]` and `[default]`
    // which must be filtered later
    let valid_profiles = validated_profiles
        .filter_map(|(name, profile)| match name {
            Ok(profile_name) => Some((profile_name, profile)),
            Err(err_str) => {
                tracing::warn!("{}", err_str);
                None
            }
        })
        .collect::<Vec<_>>();
    // if a `[profile default]` exists then we should ignore `[default]`
    let ignore_unprefixed_default = valid_profiles
        .iter()
        .any(|(profile, _)| profile.name == DEFAULT && profile.has_profile_prefix);

    for (profile_name, raw_profile) in valid_profiles {
        // When normalizing profiles, profiles should be merged. However, `[profile default]` and
        // `[default]` are considered two separate profiles. Furthermore, `[profile default]` fully
        // replaces any contents of `[default]`!
        if ignore_unprefixed_default
            && profile_name.name == DEFAULT
            && !profile_name.has_profile_prefix
        {
            tracing::warn!("profile `default` ignored because `[profile default]` was found which takes priority");
            continue;
        }
        let profile = base
            .profiles
            .entry(profile_name.name.to_string())
            .or_insert_with(|| Profile::new(profile_name.name.to_string(), Default::default()));
        merge_into_base(profile, raw_profile)
    }
}

fn merge_into_base(target: &mut Profile, profile: HashMap<&str, Cow<'_, str>>) {
    for (k, v) in profile {
        match validate_identifier(k) {
            Ok(k) => {
                target
                    .properties
                    .insert(k.to_owned(), Property::new(k.to_owned(), v.into()));
            }
            Err(_) => {
                tracing::warn!(profile = %&target.name, key = ?k, "key ignored because `{}` was not a valid identifier", k);
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
    use std::collections::HashMap;

    use tracing_test::traced_test;

    use crate::profile::parser::parse::RawProfileSet;
    use crate::profile::ProfileSet;

    use super::{merge_in, ProfileName};
    use crate::profile::parser::normalize::validate_identifier;
    use crate::profile::profile_file::ProfileFileKind;

    #[test]
    fn profile_name_parsing() {
        assert_eq!(
            ProfileName::parse("profile name"),
            ProfileName {
                name: "name",
                has_profile_prefix: true
            }
        );
        assert_eq!(
            ProfileName::parse("name"),
            ProfileName {
                name: "name",
                has_profile_prefix: false
            }
        );
        assert_eq!(
            ProfileName::parse("profile\tname"),
            ProfileName {
                name: "name",
                has_profile_prefix: true
            }
        );
        assert_eq!(
            ProfileName::parse("profile     name  "),
            ProfileName {
                name: "name",
                has_profile_prefix: true
            }
        );
        assert_eq!(
            ProfileName::parse("profilename"),
            ProfileName {
                name: "profilename",
                has_profile_prefix: false
            }
        );
        assert_eq!(
            ProfileName::parse("   whitespace   "),
            ProfileName {
                name: "whitespace",
                has_profile_prefix: false
            }
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
            out.insert("invalid key", "value".into());
            out
        });
        let mut base = ProfileSet::empty();
        merge_in(&mut base, profile, ProfileFileKind::Config);
        assert!(base
            .get_profile("default")
            .expect("contains default profile")
            .properties
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

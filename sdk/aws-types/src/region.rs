/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::borrow::Cow;
use std::env::VarError;

/// The region to send requests to.
///
/// The region MUST be specified on a request. It may be configured globally or on a
/// per-client basis unless otherwise noted. A full list of regions is found in the
/// "Regions and Endpoints" document.
///
/// See http://docs.aws.amazon.com/general/latest/gr/rande.html for
/// information on AWS regions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Region(
    // Regions are almost always known statically. However, as an escape hatch for when they
    // are not, allow for an owned region
    Cow<'static, str>,
);

impl AsRef<str> for Region {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Region {
    pub fn new(region: impl Into<Cow<'static, str>>) -> Self {
        Self(region.into())
    }
}

/// Provide a [`Region`](Region) to use with AWS requests
///
/// For most cases [`default_provider`](default_provider) will be the best option, implementing
/// a standard provider chain.
pub trait ProvideRegion: Send + Sync {
    fn region(&self) -> Option<Region>;
}

impl ProvideRegion for Region {
    fn region(&self) -> Option<Region> {
        Some(self.clone())
    }
}

impl<'a> ProvideRegion for &'a Region {
    fn region(&self) -> Option<Region> {
        Some((*self).clone())
    }
}

pub fn default_provider() -> impl ProvideRegion {
    EnvironmentProvider::new()
}

#[non_exhaustive]
pub struct EnvironmentProvider {
    env: Box<dyn Fn(&str) -> Result<String, VarError> + Send + Sync>,
}

impl Default for EnvironmentProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::redundant_closure)] // https://github.com/rust-lang/rust-clippy/issues/7218
impl EnvironmentProvider {
    pub fn new() -> Self {
        EnvironmentProvider {
            env: Box::new(|key| std::env::var(key)),
        }
    }
}

impl ProvideRegion for EnvironmentProvider {
    fn region(&self) -> Option<Region> {
        (self.env)("AWS_REGION")
            .or_else(|_| (self.env)("AWS_DEFAULT_REGION"))
            .map(Region::new)
            .ok()
    }
}

/// The region to use when signing requests
///
/// Generally, user code will not need to interact with `SigningRegion`. See `[Region](crate::Region)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SigningRegion(Cow<'static, str>);

impl AsRef<str> for SigningRegion {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<Region> for SigningRegion {
    fn from(inp: Region) -> Self {
        SigningRegion(inp.0)
    }
}

#[cfg(test)]
mod test {
    use crate::region::{EnvironmentProvider, ProvideRegion, Region};
    use std::collections::HashMap;
    use std::env::VarError;

    fn test_provider(map: HashMap<&'static str, &'static str>) -> EnvironmentProvider {
        EnvironmentProvider {
            env: Box::new(move |key: &str| {
                map.get(key)
                    .ok_or(VarError::NotPresent)
                    .map(|k| k.to_string())
            }),
        }
    }

    #[test]
    fn no_region() {
        assert_eq!(test_provider(HashMap::new()).region(), None);
    }

    #[test]
    fn prioritize_aws_region() {
        let mut env = HashMap::new();
        env.insert("AWS_REGION", "us-east-1");
        env.insert("AWS_DEFAULT_REGION", "us-east-2");
        assert_eq!(test_provider(env).region(), Some(Region::new("us-east-1")));
    }

    #[test]
    fn fallback_to_default_region() {
        let mut env = HashMap::new();
        env.insert("AWS_DEFAULT_REGION", "us-east-2");
        assert_eq!(test_provider(env).region(), Some(Region::new("us-east-2")));
    }
}

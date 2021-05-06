/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::borrow::Cow;

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
    EnvironmentProvider
}

#[non_exhaustive]
#[derive(Default)]
pub struct EnvironmentProvider;

impl EnvironmentProvider {
    pub fn new() -> Self {
        EnvironmentProvider
    }
}

impl ProvideRegion for EnvironmentProvider {
    fn region(&self) -> Option<Region> {
        std::env::var("AWS_DEFAULT_REGION").map(Region::new).ok()
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

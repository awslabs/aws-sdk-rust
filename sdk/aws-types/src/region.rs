/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Region type for determining the endpoint to send requests to.

use aws_smithy_types::config_bag::{Storable, StoreReplace};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};

/// The region to send requests to.
///
/// The region MUST be specified on a request. It may be configured globally or on a
/// per-client basis unless otherwise noted. A full list of regions is found in the
/// "Regions and Endpoints" document.
///
/// See <http://docs.aws.amazon.com/general/latest/gr/rande.html> for
/// information on AWS regions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

impl Display for Region {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for Region {
    type Storer = StoreReplace<Region>;
}

impl Region {
    /// Creates a new `Region` from the given string.
    pub fn new(region: impl Into<Cow<'static, str>>) -> Self {
        Self(region.into())
    }

    /// Const function that creates a new `Region` from a static str.
    pub const fn from_static(region: &'static str) -> Self {
        Self(Cow::Borrowed(region))
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

impl From<&'static str> for SigningRegion {
    fn from(region: &'static str) -> Self {
        Self::from_static(region)
    }
}

impl SigningRegion {
    /// Creates a `SigningRegion` from a static str.
    pub const fn from_static(region: &'static str) -> Self {
        SigningRegion(Cow::Borrowed(region))
    }
}

impl Storable for SigningRegion {
    type Storer = StoreReplace<Self>;
}

/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::tristate::TriState;
use std::time::Duration;

/// TCP timeouts used by lower-level `DynConnector`s
#[non_exhaustive]
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Tcp {
    connect: TriState<Duration>,
    write: TriState<Duration>,
    read: TriState<Duration>,
}

impl Tcp {
    /// Create a new TCP timeout config with no timeouts set
    pub fn new() -> Self {
        Default::default()
    }

    /// Return true if any timeouts are intentionally set or disabled
    pub fn has_timeouts(&self) -> bool {
        !self.is_unset()
    }

    /// Return true if all timeouts are unset
    fn is_unset(&self) -> bool {
        self.connect.is_unset() && self.write.is_unset() && self.read.is_unset()
    }

    /// Merges two TCP timeout configs together.
    pub fn take_unset_from(self, other: Self) -> Self {
        Self {
            connect: self.connect.or(other.connect),
            write: self.write.or(other.write),
            read: self.read.or(other.read),
        }
    }
}

impl From<super::Config> for Tcp {
    fn from(config: super::Config) -> Self {
        config.tcp
    }
}

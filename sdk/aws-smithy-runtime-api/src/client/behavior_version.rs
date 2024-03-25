/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Behavior Major version of the client

/// Behavior major-version of the client
///
/// Over time, new best-practice behaviors are introduced. However, these behaviors might not be
/// backwards compatible. For example, a change which introduces new default timeouts or a new
/// retry-mode for all operations might be the ideal behavior but could break existing applications.
#[derive(Clone)]
pub struct BehaviorVersion {
    _private: (),
}

impl BehaviorVersion {
    /// This method will always return the latest major version.
    ///
    /// This is the recommend choice for customers who aren't reliant on extremely specific behavior
    /// characteristics. For example, if you are writing a CLI app, the latest behavior major
    /// version is probably the best setting for you.
    ///
    /// If, however, you're writing a service that is very latency sensitive, or that has written
    /// code to tune Rust SDK behaviors, consider pinning to a specific major version.
    ///
    /// The latest version is currently [`BehaviorVersion::v2023_11_09`]
    pub fn latest() -> Self {
        Self::v2023_11_09()
    }

    /// This method returns the behavior configuration for November 9th, 2023
    ///
    /// When a new behavior major version is released, this method will be deprecated.
    pub fn v2023_11_09() -> Self {
        Self { _private: () }
    }
}

impl std::fmt::Debug for BehaviorVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BehaviorVersion")
            .field("name", &"v2023_11_09")
            .finish()
    }
}

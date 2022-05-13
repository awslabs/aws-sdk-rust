/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Contains [`TriState`](TriState) definition and impls

/// Utility for tracking set vs. unset vs explicitly disabled
///
/// If someone explicitly disables something, we don't need to warn them that it may be missing. This
/// enum impls `From`/`Into` `Option<T>` for ease of use.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TriState<T> {
    /// This variant represents something that was unset by default
    Unset,
    /// This variant represents something that was intentionally unset
    Disabled,
    /// This variant represents something that was intentionally set
    Set(T),
}

impl<T> TriState<T> {
    /// Create a TriState, returning `Unset` when `None` is passed
    pub fn or_unset(t: Option<T>) -> Self {
        match t {
            Some(t) => Self::Set(t),
            None => Self::Unset,
        }
    }

    /// Return `true` if this `TriState` is `Unset`
    pub fn is_unset(&self) -> bool {
        matches!(self, TriState::Unset)
    }

    /// Returns the tristate if it contains a set value or is disabled, otherwise returns `other`
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::time::Duration;
    /// # use aws_smithy_types::tristate::TriState;
    /// let disabled_timeout: TriState<Duration> = TriState::Disabled;
    /// let timeout: TriState<Duration> = TriState::Set(Duration::from_secs(1));
    /// assert_eq!(timeout.or(disabled_timeout), TriState::Set(Duration::from_secs(1)));
    ///
    /// let disabled_timeout: TriState<Duration> = TriState::Disabled;
    /// let timeout: TriState<Duration> = TriState::Set(Duration::from_secs(2));
    /// assert_eq!(disabled_timeout.or(timeout), TriState::Disabled);
    ///
    /// let unset_timeout: TriState<Duration> = TriState::Unset;
    /// let timeout: TriState<Duration> = TriState::Set(Duration::from_secs(3));
    /// assert_eq!(unset_timeout.or(timeout), TriState::Set(Duration::from_secs(3)));
    /// ```
    pub fn or(self, other: TriState<T>) -> TriState<T> {
        use TriState::*;

        match self {
            Set(_) | Disabled => self,
            Unset => other,
        }
    }

    /// Maps a `TriState<T>` to `TriState<U>` by applying a function to a contained value.
    pub fn map<U, F>(self, f: F) -> TriState<U>
    where
        F: FnOnce(T) -> U,
    {
        use TriState::*;

        match self {
            Set(x) => Set(f(x)),
            Unset => Unset,
            Disabled => Disabled,
        }
    }
}

impl<T> Default for TriState<T> {
    fn default() -> Self {
        Self::Unset
    }
}

impl<T> From<Option<T>> for TriState<T> {
    fn from(t: Option<T>) -> Self {
        match t {
            Some(t) => TriState::Set(t),
            None => TriState::Disabled,
        }
    }
}

impl<T> From<TriState<T>> for Option<T> {
    fn from(t: TriState<T>) -> Self {
        match t {
            TriState::Disabled | TriState::Unset => None,
            TriState::Set(t) => Some(t),
        }
    }
}

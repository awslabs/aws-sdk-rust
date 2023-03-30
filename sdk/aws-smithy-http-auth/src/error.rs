/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP Auth Error

use std::cmp::PartialEq;
use std::fmt::Debug;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum AuthErrorKind {
    InvalidLocation,
    MissingRequiredField(&'static str),
    SchemeNotAllowed,
}

/// Error for Smithy authentication
#[derive(Debug, Eq, PartialEq)]
pub struct AuthError {
    kind: AuthErrorKind,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AuthErrorKind::*;
        match &self.kind {
            InvalidLocation => write!(f, "invalid location: expected `header` or `query`"),
            MissingRequiredField(field) => write!(f, "missing required field: {}", field),
            SchemeNotAllowed => write!(
                f,
                "scheme only allowed when it is set into the `Authorization` header"
            ),
        }
    }
}

impl From<AuthErrorKind> for AuthError {
    fn from(kind: AuthErrorKind) -> Self {
        Self { kind }
    }
}
